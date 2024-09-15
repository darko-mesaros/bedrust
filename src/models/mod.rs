pub mod converse;
pub mod converse_stream;

use anyhow::{anyhow, Result};
use aws_sdk_bedrock::{
    self,
    types::{FoundationModelDetails, ModelModality},
};

pub enum ModelFeatures {
    Streaming,
    Images,
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
    feature: ModelFeatures,
) -> Result<bool, anyhow::Error> {
    let call = c.get_foundation_model().model_identifier(m);
    let res = call.send().await;
    let model_details: FoundationModelDetails = res?
        .model_details()
        .ok_or_else(|| anyhow!("Unable to get model details"))?
        .clone();

    match feature {
        ModelFeatures::Images => match model_details.input_modalities {
            Some(o) => {
                if o.contains(&ModelModality::Image) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        },
        ModelFeatures::Streaming => match model_details.response_streaming_supported {
            Some(o) => Ok(o),
            None => Ok(false),
        },
    }
}
