mod config;
mod prompt_generator;
mod http_client;
mod benchmark;
mod output;

use config::BenchmarkConfig;
use prompt_generator::PromptGenerator;
use http_client::HttpClient;
use benchmark::BenchmarkRunner;
use output::OutputWriter;
use std::env;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <config_file>", args[0]);
        eprintln!("Example: {} configs/scenario-1.yaml", args[0]);
        std::process::exit(1);
    }
    
    let config_file = &args[1];
    
    println!("Loading configuration from: {}", config_file);
    let config = BenchmarkConfig::from_file(config_file)?;
    
    println!("Configuration loaded:");
    println!("  Model: {}", config.model);
    println!("  Base URL: {}", config.base_url);
    println!("  Runs: {}", config.runs);
    println!("  Concurrent requests: {}", config.num_concurrent_requests);
    println!("  Test cases: {}", config.cases.len());
    
    // Phase 1: Generate prompts
    println!("\n=== Phase 1: Generating prompts ===");
    let prompt_generator = PromptGenerator::new("tokenizers/tokenizer.json")?;
    
    let prompts = prompt_generator
        .generate_prompts(&config.cases, config.runs)
        .await?;
    
    println!("Prompts generated for {} cases", prompts.len());
    
    // Phase 2: Run benchmarks
    println!("\n=== Phase 2: Running benchmarks ===");
    let benchmark_runner = BenchmarkRunner::new(config.clone());
    let results = benchmark_runner.run_benchmarks(&prompts).await?;
    
    println!("Benchmark completed with {} results", results.len());
    
    // Phase 3: Write results
    println!("\n=== Phase 3: Writing results ===");
    let output_writer = OutputWriter::new(
        config.output_dir.clone(),
        config.output_prefix_name.clone(),
    );
    
    match config.save_format.as_str() {
        "csv" => {
            output_writer.write_csv(&results)?;
        }
        "jsonl" => {
            output_writer.write_jsonl(&results)?;
        }
        _ => {
            // Default to CSV
            output_writer.write_csv(&results)?;
        }
    }
    
    println!("\nBenchmark completed successfully!");
    
    // Print summary
    println!("\n=== Summary ===");
    for result in &results {
        println!("Case ({}→{}): TTFT={:.2}ms, TPOT={:.2}ms, TPS={:.2}", 
            result.input_tokens, 
            result.output_tokens,
            result.ttft_ms,
            result.tpot_ms,
            result.throughput_tps
        );
    }
    
    Ok(())
}
