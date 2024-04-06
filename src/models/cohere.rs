use aws_sdk_bedrockruntime::primitives::Blob;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CohereCommandConfig {
    pub max_tokens: i32,
    pub temperature: f32,
    pub p: f32,
    pub k: i32,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

#[derive(serde::Serialize, Debug)]
pub struct CohereBody {
    pub prompt: String,
    pub max_tokens: i32,
    pub temperature: f32,
    pub p: f32,
    pub k: i32,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

impl CohereBody {
    pub fn new(
        prompt: String,
        max_tokens: i32,
        temperature: f32,
        p: f32,
        k: i32,
        stop_sequences: Vec<String>,
        stream: bool,
    ) -> CohereBody {
        CohereBody {
            prompt,
            max_tokens,
            temperature,
            p,
            k,
            stop_sequences,
            stream,
        }
    }
}

pub(crate) trait Blobbable {
    fn to_blob(&self) -> Blob;
}

impl<T> Blobbable for T
where
    T: Serialize,
{
    fn to_blob(&self) -> Blob {
        let blob_string =
            serde_json::to_vec(&self).expect("unserializable settings. this is a bug.");
        Blob::new(blob_string)
    }
}

#[derive(serde::Deserialize)]
pub struct CohereResponseGenerations {
    pub generations: Vec<CohereResponseText>,
}
#[derive(serde::Deserialize, Debug)]
pub struct CohereResponseText {
    pub text: String,
}
