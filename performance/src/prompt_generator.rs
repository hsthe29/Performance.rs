use rand::Rng;
use std::collections::HashMap;
use anyhow::{Result};
use crate::config::TestCase;
use tokenizers::Tokenizer;

pub struct PromptGenerator {
   tokenizer: Tokenizer,
}

impl PromptGenerator {
    pub fn new(input_file: &str) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(input_file)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer from file {}: {}", input_file, e))?;
        Ok(PromptGenerator {
            tokenizer
        })
    }

    pub fn generate_tokens(&self, low: usize, high: usize, size: usize) -> Vec<u32> {
        let mut rng = rand::rng();

        let tokens_buffer: Vec<u32> = (0..size)
            .map(|_| rng.random_range(low..=high) as u32)
            .collect();
        tokens_buffer
    }

    pub async fn generate_prompts(
        &self,
        cases: &[TestCase],
        runs: u32,
    ) -> Result<HashMap<(u32, u32), Vec<String>>> {
        let mut prompts = HashMap::new();
        
        for case in cases {
            let mut case_prompts = Vec::new();
            
            for run in 0..runs {
                println!("Generating prompt for case (input:{}, output:{}) run {}/{}", 
                    case.input_tokens, case.output_tokens, run + 1, runs);
                
                
                let prepared_prompt = self.prepare_prompt(case.input_tokens as usize)?;
                case_prompts.push(prepared_prompt);
            }
            
            prompts.insert((case.input_tokens, case.output_tokens), case_prompts);
        }
        
        Ok(prompts)
    }

    fn prepare_prompt(
        &self,
        target_tokens: usize,
    ) -> Result<String> {
        let tokens = self.generate_tokens(
            100,
            50_000,
            target_tokens,
        );

        let prompt = self.tokenizer
            .decode(&tokens, true)
            .map_err(|e| anyhow::anyhow!("Failed to decode ids to string: {:?}", e))?;

        Ok(prompt)
    }
}
