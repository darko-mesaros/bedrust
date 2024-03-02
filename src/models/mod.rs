pub mod llama2;
pub mod cohere;
pub mod claude;
pub mod jurrasic2;
pub mod titan;
pub mod mistral;

use llama2::Llama270bConfig;
use cohere::CohereCommandConfig;
use claude::{ClaudeV2Config, ClaudeV21Config};
use jurrasic2::Jurrasic2UltraConfig;
use titan::TitanTextExpressV1Config;
use mistral::{Mistral7bInstruct, Mixtral8x7bInstruct};

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
    pub titan_text_express_v1: TitanTextExpressV1Config,
    pub mixtral_8x7b_instruct: Mixtral8x7bInstruct,
    pub mistral_7b_instruct: Mistral7bInstruct,
}

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
