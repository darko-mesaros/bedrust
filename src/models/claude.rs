use anyhow::Result;
use aws_sdk_bedrockruntime::primitives::Blob;
use serde::{Deserialize, Serialize};

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

#[derive(serde::Serialize, Debug)]
pub struct ClaudeBody {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens_to_sample: i32,
    pub stop_sequences: Vec<String>,
}

impl ClaudeBody {
    pub fn new(
        prompt: String,
        temperature: f32,
        top_p: f32,
        top_k: i32,
        max_tokens_to_sample: i32,
        stop_sequences: Vec<String>,
    ) -> ClaudeBody {
        ClaudeBody {
            prompt,
            temperature,
            top_p,
            top_k,
            max_tokens_to_sample,
            stop_sequences,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string.clone());
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ClaudeResponse {
    pub completion: String,
}
