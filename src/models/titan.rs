use serde::{Serialize, Deserialize};
use aws_sdk_bedrockruntime::primitives::Blob;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct TitanTextExpressV1Config {
    pub temperature: f32,
    pub top_p: f32,
    pub max_token_count: i32,
    pub stop_sequences: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TitanTextV1Body {
    pub input_text: String,
    pub text_generation_config: TitanTextV1textGenerationConfig
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TitanTextV1textGenerationConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub max_token_count: i32,
    pub stop_sequences: Vec<String>,
}

impl TitanTextV1Body {
    pub fn new(input_text: String, temperature: f32, top_p: f32, max_token_count: i32, stop_sequences: Vec<String>) -> TitanTextV1Body {
        let text_gen_config = TitanTextV1textGenerationConfig {
            temperature,
            top_p,
            max_token_count,
            stop_sequences
        };
        TitanTextV1Body {
            input_text,
            text_generation_config: text_gen_config
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize)]
pub struct TitanTextV1Response {
   pub results: Vec<TitanTextV1Results>
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TitanTextV1Results {
   pub output_text: String,
}
