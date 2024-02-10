use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_types::region::Region;
use std::env;

use anyhow::Result;

use aws_sdk_bedrockruntime::{Client, Error};
use aws_sdk_bedrockruntime::primitives::Blob;

use serde_json::{Value};

//========================================
pub struct BedrockCall {
    pub body: Blob,
    pub content_type: String,
    pub accept: String,
    pub model_id: String,
}

impl BedrockCall {
    pub fn new(body: Blob, content_type: String, accept: String, model_id: String ) -> BedrockCall {
        BedrockCall {
            body,
            content_type,
            accept,
            model_id,
        }
    }
}

pub struct CohereBody {
    pub prompt: String,
    pub max_tokens: String,
    pub temperature: String,
    pub p: String,
    pub k: String,
}

impl CohereBody {
    pub fn new(prompt: String, mt: String, temp: String, p: String, k: String) -> CohereBody {
        CohereBody {
            prompt,
            max_tokens: mt,
            temperature: temp,
            p,
            k,
        }
    }

    pub fn convert_to_blob(&self) -> Blob {
        let blob_string = format!(r#"{{"prompt":"{}","max_tokens":{},"temperature":{},"p":{},"k":{}}}"#, &self.prompt, &self.max_tokens, &self.temperature, &self.p, &self.k);

        let body: Blob = Blob::new(blob_string);
        body
    }
}
//========================================


pub fn convert_json(s: &str) -> Result<String> {
    let v: Value = serde_json::from_str(s)?;
    // future_highway: we convert to Option<&str>
    // then using .ok_or() we get the &str
    // then to get the String we use .to_string()
    let response = v["generations"][0]["text"]
        .as_str()
        .ok_or(anyhow::anyhow!("Not really a string"))?
        .to_string();
    Ok(response)
}

pub async fn configure_aws(s: String) -> aws_config::SdkConfig {
    let provider = RegionProviderChain::first_try(env::var("AWS_DEFAULT_REGION").ok().map(Region::new))
        .or_default_provider()
        .or_else(Region::new(s));

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(provider)
        .load()
        .await;

    config

}

pub async fn call_bedrock(bc: Client, c: BedrockCall) -> Result<String>{

    let response = bc.invoke_model()
    .body(c.body)
    .content_type(c.content_type)
    .accept(c.accept)
    .model_id(c.model_id)
    .send()
    .await?;

    let response_body = response
        .body
        .into_inner();

    let reponse_string = String::from_utf8(response_body)?;
    Ok(reponse_string)

}
