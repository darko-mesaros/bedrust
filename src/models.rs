use serde::{Serialize, Deserialize};
use aws_sdk_bedrock::{self,
types::FoundationModelDetails,
};
use anyhow::Result;

use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigs {
    pub llama270b: Llama270bConfig,
    pub cohere_command: CohereCommandConfig,
    pub claude_v2: ClaudeV2Config,
    pub claude_v21: ClaudeV21Config,
    pub jurrasic_2_ultra: Jurrasic2UltraConfig,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeV21Config {
    pub temperature: f32,
    pub p: f32,
    pub k: i32,
    pub max_tokens_to_sample: i32,
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Jurrasic2UltraConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: i32,
    pub stop_sequences: Vec<String>,
}

//======================================== FUNCTIONS
pub fn load_config(f: String) -> Result<ModelConfigs> {
    let file = fs::File::open(f)?;
    let config: ModelConfigs = ron::de::from_reader(file)?;
    Ok(config)
}

pub async fn check_for_streaming(m: String, c: aws_sdk_bedrock::Client) -> Result<bool> {
    let call = c.get_foundation_model()
                    .model_identifier(m);
    let res = call.send().await;
    let model_details: FoundationModelDetails = res.unwrap().model_details().unwrap().clone();

    match model_details.response_streaming_supported {
        Some(o) => Ok(o),
        None => Ok(false),
    }
}

