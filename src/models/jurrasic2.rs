use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Jurrasic2UltraConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: i32,
    pub stop_sequences: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Jurrasic2Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: i32,
    pub stop_sequences: Vec<String>,
}

impl Jurrasic2Body {
    pub fn new(
        prompt: String,
        temperature: f32,
        top_p: f32,
        max_tokens: i32,
        stop_sequences: Vec<String>,
    ) -> Jurrasic2Body {
        Jurrasic2Body {
            prompt,
            temperature,
            top_p,
            max_tokens,
            stop_sequences,
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Jurrasic2ResponseCompletions {
    pub completions: Vec<Jurrasic2ResponseData>,
}
#[derive(serde::Deserialize, Debug)]
pub struct Jurrasic2ResponseData {
    pub data: Jurrasic2ResponseText,
}

#[derive(serde::Deserialize, Debug)]
pub struct Jurrasic2ResponseText {
    pub text: String,
}
