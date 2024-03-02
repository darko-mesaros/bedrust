use serde::{Serialize, Deserialize};
use aws_sdk_bedrockruntime::primitives::Blob;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Mixtral8x7bInstruct {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub stop: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Mistral7bInstruct {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub stop: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
pub struct Mistral7Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub stop: Vec<String>,
}

impl Mistral7Body {
    pub fn new(prompt: String, temperature: f32, top_p: f32, top_k: i32, max_tokens: i32, stop: Vec<String>) -> Mistral7Body {
        Mistral7Body {
            prompt,
            temperature,
            top_p,
            top_k,
            max_tokens,
            stop,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Mistral7Results {
   pub outputs: Vec<Mistral7Outputs>
}

#[derive(serde::Deserialize, Debug)]
pub struct Mistral7Outputs {
   pub text: String,
}
