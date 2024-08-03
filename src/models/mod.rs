pub mod claude;
pub mod claudev3;
pub mod cohere;
pub mod jurrasic2;
pub mod llama2;
pub mod mistral;
pub mod titan;
pub mod converse;
pub mod converse_stream;

use claude::{ClaudeV21Config, ClaudeV2Config};
use claudev3::ClaudeV3Config;
use cohere::CohereCommandConfig;
use jurrasic2::Jurrasic2UltraConfig;
use llama2::Llama270bConfig;
use mistral::{Mistral7bInstruct, Mixtral8x7bInstruct, MistralLarge};
use titan::TitanTextExpressV1Config;

use anyhow::{anyhow, Result};
use aws_sdk_bedrock::{self, types::{FoundationModelDetails, ModelModality}};
use serde::{Deserialize, Serialize};

use std::fs;

use dirs::home_dir;

use crate::constants;

pub enum ModelFeatures {
    Streaming,
    Images,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigs {
    pub llama270b: Llama270bConfig,
    pub cohere_command: CohereCommandConfig,
    pub claude_v2: ClaudeV2Config,
    pub claude_v21: ClaudeV21Config,
    pub claude_v3: ClaudeV3Config,
    pub jurrasic_2_ultra: Jurrasic2UltraConfig,
    pub titan_text_express_v1: TitanTextExpressV1Config,
    pub mixtral_8x7b_instruct: Mixtral8x7bInstruct,
    pub mistral_7b_instruct: Mistral7bInstruct,
    pub mistral_large: MistralLarge,
}

pub fn load_model_config() -> Result<ModelConfigs> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    let model_config_file_path = config_dir.join(constants::MODEL_CONFIG_FILE_NAME);
    let file = fs::File::open(model_config_file_path)?;
    let config: ModelConfigs = ron::de::from_reader(file)?;
    Ok(config)
}

pub async fn check_for_streaming(
    m: String,
    c: &aws_sdk_bedrock::Client,
) -> Result<bool, anyhow::Error> {
    let call = c.get_foundation_model().model_identifier(m);
    let res = call.send().await;
    let model_details: FoundationModelDetails = res?
        .model_details()
        .ok_or_else(|| anyhow!("Unable to get model details"))?
        .clone();

    match model_details.response_streaming_supported {
        Some(o) => Ok(o),
        None => Ok(false),
    }
}

pub async fn check_model_features(
    m: &str,
    c: &aws_sdk_bedrock::Client,
    feature: ModelFeatures
) -> Result<bool, anyhow::Error> {
    let call = c.get_foundation_model().model_identifier(m);
    let res = call.send().await;
    let model_details: FoundationModelDetails = res?
        .model_details()
        .ok_or_else(|| anyhow!("Unable to get model details"))?
        .clone();

    match feature {
        ModelFeatures::Images => {
            match model_details.input_modalities{
                Some(o) => {
                    if o.contains(&ModelModality::Image) {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                None => Ok(false),
            }
        }
        ModelFeatures::Streaming => {
            match model_details.response_streaming_supported {
                Some(o) => Ok(o),
                None => Ok(false),
            }
        }
    }
}
