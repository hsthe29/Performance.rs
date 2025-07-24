use std::collections::HashMap;
use futures::future::join_all;
use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::config::{BenchmarkConfig, TestCase};
use crate::http_client::{HttpClient, RequestMetrics};
use crate::output::BenchmarkResult;

pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    client: HttpClient,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        let client = HttpClient::new(
            config.base_url.clone(),
            config.api_key.clone(),
            config.model.clone(),
        );
        
        BenchmarkRunner { config, client }
    }

    pub async fn run_benchmarks(
        &self,
        prompts: &HashMap<(u32, u32), Vec<String>>,
    ) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();
        
        for case in &self.config.cases {
            println!("Running benchmark for case: input_tokens={}, output_tokens={}", 
                case.input_tokens, case.output_tokens);
            
            let case_prompts = prompts
                .get(&(case.input_tokens, case.output_tokens))
                .ok_or_else(|| anyhow::anyhow!("No prompts found for case"))?;
            
            for run in 0..self.config.runs {
                println!("  Run {}/{}", run + 1, self.config.runs);
                
                let prompt = &case_prompts[run as usize];
                let run_result = self.run_single_benchmark(case, prompt).await?;
                results.push(run_result);
            }
        }
        
        Ok(results)
    }

    async fn run_single_benchmark(
        &self,
        case: &TestCase,
        prompt: &str,
    ) -> Result<BenchmarkResult> {
        let tasks: Vec<_> = (0..self.config.num_concurrent_requests)
            .map(|_| {
                let client = &self.client;
                let prompt = prompt.to_string();
                let max_tokens = case.output_tokens;
                async move {
                    client.benchmark_request_with_retry(&prompt, max_tokens, 3).await
                }
            })
            .collect();

        let metrics_results: Vec<Result<RequestMetrics>> = join_all(tasks).await;
        
        // Extract successful metrics
        let mut successful_metrics = Vec::new();
        let mut failed_count = 0;
        
        for result in metrics_results {
            match result {
                Ok(metrics) => successful_metrics.push(metrics),
                Err(e) => {
                    eprintln!("Request failed: {}", e);
                    failed_count += 1;
                }
            }
        }
        
        if successful_metrics.is_empty() {
            return Err(anyhow::anyhow!("All requests failed"));
        }
        
        if failed_count > 0 {
            println!("  Warning: {} requests failed", failed_count);
        }

        // Calculate aggregate metrics
        let avg_ttft = successful_metrics.iter().map(|m| m.ttft_ms).sum::<f64>() 
            / successful_metrics.len() as f64;
        
        let avg_tpot = successful_metrics.iter().map(|m| m.tpot_ms).sum::<f64>() 
            / successful_metrics.len() as f64;
        
        // Throughput = 1000/TPOT * num_concurrent_requests
        let throughput_tps = if avg_tpot > 0.0 {
            (1000.0 / avg_tpot) * successful_metrics.len() as f64
        } else {
            0.0
        };

        Ok(BenchmarkResult {
            timestamp: Utc::now(),
            model: self.config.model.clone(),
            input_tokens: case.input_tokens.clone(),
            output_tokens: case.output_tokens.clone(),
            runs: self.config.runs,
            num_concurrent_requests: successful_metrics.len() as u32,
            ttft_ms: avg_ttft,
            tpot_ms: avg_tpot,
            throughput_tps,
        })
    }
}
