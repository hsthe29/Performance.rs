use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};
use anyhow::Result;
use futures::StreamExt;

#[derive(Debug, Clone, Serialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub stream: bool,
    pub stop: Option<String>,
    pub stream_options: Option<StreamOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamOptions {
    pub include_usage: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub text: String,
    pub index: u32,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub ttft_ms: f64,
    pub tpot_ms: f64,
    pub total_tokens: u32,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub struct HttpClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl HttpClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        
        HttpClient {
            client,
            base_url,
            api_key,
            model,
        }
    }

    pub async fn generate_for_tokens(&self, prompt: &str, max_tokens: u32) -> Result<CompletionResponse> {
        let request = CompletionRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            max_tokens,
            temperature: 0.7,
            stream: false,
            stop: None,
            stream_options: None,
        };

        let response = self.client
            .post(&format!("{}/completions", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let completion: CompletionResponse = response.json().await?;
        Ok(completion)
    }

    pub async fn benchmark_request_with_retry(
        &self,
        prompt: &str,
        max_tokens: u32,
        max_retries: u32,
    ) -> Result<RequestMetrics> {
        for attempt in 0..=max_retries {
            match self.benchmark_request(prompt, max_tokens).await {
                Ok(metrics) => return Ok(metrics),
                Err(e) if attempt < max_retries => {
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn benchmark_request(&self, prompt: &str, max_tokens: u32) -> Result<RequestMetrics> {
        let request = CompletionRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            max_tokens: max_tokens.clone(),
            temperature: 0.7,
            stream: true,
            stop: None,
            stream_options: Some(StreamOptions {
                include_usage: true,
            }),
        };

        let start_time = Instant::now();
        
        let response = self.client
            .post(&format!("{}/completions", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut first_token_time: Option<Instant> = None;
        let mut last_token_time = start_time;
        let mut token_intervals = Vec::new();
        let mut usage_info: Option<Usage> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..]; // Remove "data: " prefix
                    
                    if data == "[DONE]" {
                        break;
                    }
                    
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        let current_time = Instant::now();
                        
                        // Check if this chunk has content (indicates a token)
                        if let Some(choices) = parsed["choices"].as_array() {
                            if let Some(choice) = choices.first() {
                                if let Some(text) = choice["text"].as_str() {
                                    if !text.is_empty() {
                                        if first_token_time.is_none() {
                                            first_token_time = Some(current_time);
                                        } else {
                                            let interval = current_time.duration_since(last_token_time).as_millis() as f64;
                                            token_intervals.push(interval);
                                        }
                                        last_token_time = current_time;
                                    }
                                }
                            }
                        }
                        
                        // Check for usage information
                        if let Some(usage) = parsed["usage"].as_object() {
                            usage_info = Some(Usage {
                                prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                                completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                                total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
                            });
                        }
                    }
                }
            }
        }

        // let usage = usage_info.ok_or_else(|| anyhow::anyhow!("No usage information received"))?;
        let usage = usage_info.unwrap_or(
            Usage { prompt_tokens: 0u32, completion_tokens: max_tokens.clone(), total_tokens: max_tokens.clone() }
        );
        
        let ttft_ms = first_token_time
            .map(|t| t.duration_since(start_time).as_millis() as f64)
            .unwrap_or(0.0);
        
        let tpot_ms = if token_intervals.is_empty() {
            0.0
        } else {
            token_intervals.iter().sum::<f64>() / token_intervals.len() as f64
        };

        Ok(RequestMetrics {
            ttft_ms,
            tpot_ms,
            total_tokens: usage.total_tokens,
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        })
    }
}
