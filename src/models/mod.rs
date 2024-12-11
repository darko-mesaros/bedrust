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
    // FIX: TEMPORARY SOLUTION DUE TO CROSS REGION INFERENCE
    // The issue here is that the converse model requires the `us.` in front of the nova models due
    // to cross-region inference. But the get_foundation_model method needs the actual model id. So
    // we are just hardcoding this in.
    // PLEASE FIX THIS FUTURE DARKO
    let model_id = match m {
        "us.amazon.nova-micro-v1:0" => "amazon.nova-micro-v1:0",
        "us.amazon.nova-lite-v1:0" => "amazon.nova-lite-v1:0",
        "us.amazon.nova-pro-v1:0" => "amazon.nova-pro-v1:0",
        _ => m
    };

    let call = c.get_foundation_model().model_identifier(model_id);
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
