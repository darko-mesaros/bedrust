use serde::{Serialize, Deserialize};
use aws_sdk_bedrockruntime::primitives::Blob;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Llama270bConfig {
    pub temperature: f32,
    pub p: f32,
    pub max_gen_len: i32,
}

#[derive(serde::Serialize, Debug)]
pub struct Llama2Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_gen_len: i32,
}

impl Llama2Body {
    pub fn new(prompt: String, temperature: f32, top_p: f32, max_gen_len: i32) -> Llama2Body {
        Llama2Body {
            prompt,
            temperature,
            top_p,
            max_gen_len,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Llama2Response {
    pub generation: String,
}
