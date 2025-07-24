use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub output_prefix_name: String,
    pub output_dir: String,
    pub save_format: String,
    pub stream_writing: bool,
    pub runs: u32,
    pub num_concurrent_requests: u32,
    pub cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl BenchmarkConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: BenchmarkConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
