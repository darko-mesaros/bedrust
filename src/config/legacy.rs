use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::ArgModels;

#[derive(Debug, Deserialize, Serialize)]
pub struct LegacyBedrustConfig {
    pub aws_profile: String,
    pub supported_images: Vec<String>,
    pub caption_prompt: String,
    pub default_model: Option<ArgModels>,
    #[serde(default = "_default_true")]
    pub show_banner: bool,
    pub inference_params: LegacyInferenceParams,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LegacyInferenceParams {
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
}

const fn _default_true() -> bool {
    true
}

pub fn load_legacy_config(path: &PathBuf) -> Result<LegacyBedrustConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: LegacyBedrustConfig = ron::from_str(&content)?;
    Ok(config)
}
