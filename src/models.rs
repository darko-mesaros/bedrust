use serde::{Serialize, Deserialize};
use anyhow::Result;

use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigs {
    pub llama270b: Llama270bConfig,
    pub cohere_command: CohereCommandConfig,
    pub claude_v2: ClaudeV2Config,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Llama270bConfig {
    pub temperature: f32,
    pub p: f32,
    pub max_gen_len: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CohereCommandConfig {
    pub max_tokens: i32,
    pub temperature: f32,
    pub p: f32,
    pub k: i32,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeV2Config {
    pub temperature: f32,
    pub p: f32,
    pub k: i32,
    pub max_tokens_to_sample: i32,
    pub stop_sequences: Vec<String>,
}

pub fn load_config(f: String) -> Result<ModelConfigs> {
    let file = fs::File::open(f)?;
    let config: ModelConfigs = ron::de::from_reader(file)?;
    Ok(config)
}

