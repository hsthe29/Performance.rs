use chrono::{DateTime, Utc};
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    #[serde(rename = "@timestamp")]
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub runs: u32,
    pub num_concurrent_requests: u32,
    #[serde(rename = "TTFT (ms)")]
    pub ttft_ms: f64,
    #[serde(rename = "TPOT (ms)")]
    pub tpot_ms: f64,
    #[serde(rename = "Throughput (TPS)")]
    pub throughput_tps: f64,
}

pub struct OutputWriter {
    output_dir: String,
    output_prefix: String,
}

impl OutputWriter {
    pub fn new(output_dir: String, output_prefix: String) -> Self {
        OutputWriter {
            output_dir,
            output_prefix,
        }
    }

    pub fn write_csv(&self, results: &[BenchmarkResult]) -> Result<String> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;
        
        // Generate filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.csv", self.output_prefix, timestamp);
        let filepath = Path::new(&self.output_dir).join(&filename);
        
        // Write CSV
        let file = File::create(&filepath)?;
        let mut writer = Writer::from_writer(file);
        
        for result in results {
            writer.serialize(result)?;
        }
        
        writer.flush()?;
        
        println!("Results written to: {}", filepath.display());
        Ok(filepath.to_string_lossy().to_string())
    }

    pub fn write_jsonl(&self, results: &[BenchmarkResult]) -> Result<String> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;
        
        // Generate filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.jsonl", self.output_prefix, timestamp);
        let filepath = Path::new(&self.output_dir).join(&filename);
        
        // Write JSONL
        let mut content = String::new();
        for result in results {
            let json_line = serde_json::to_string(result)?;
            content.push_str(&json_line);
            content.push('\n');
        }
        
        fs::write(&filepath, content)?;
        
        println!("Results written to: {}", filepath.display());
        Ok(filepath.to_string_lossy().to_string())
    }
}
