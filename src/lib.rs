pub mod captioner;
pub mod constants;
pub mod models;
pub mod utils;

use anyhow::anyhow;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_types::region::Region;
use core::panic;
use models::cohere::Blobbable;
use serde::ser::Error;
use serde::Deserialize;
use serde_json::Value;

use std::{env, io};

use anyhow::Result;

use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::types::ResponseStream;

use models::check_for_streaming;
use models::load_model_config;

use models::claude::{ClaudeBody, ClaudeResponse};
use models::claudev3::{ClaudeImageSource, ClaudeV3Body, ClaudeV3Response};
use models::cohere::{CohereBody, CohereResponseText};
use models::jurrasic2::{Jurrasic2Body, Jurrasic2ResponseCompletions};
use models::llama2::{Llama2Body, Llama2Response};
use models::mistral::{Mistral7Body, Mistral7Results};
use models::titan::{TitanTextV1Body, TitanTextV1Results};

use std::io::Write;

use crate::captioner::Image;

//======================================== AWS
pub async fn configure_aws(fallback_region: String, profile_name: String) -> aws_config::SdkConfig {
    let region_provider =
        // NOTE: this is different than the default Rust SDK behavior which checks AWS_REGION first. Is this intentional?
        RegionProviderChain::first_try(env::var("AWS_DEFAULT_REGION").ok().map(Region::new))
            .or_default_provider()
            .or_else(Region::new(fallback_region));

    aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(
            aws_config::profile::ProfileFileCredentialsProvider::builder()
                .profile_name(profile_name)
                .build(),
        )
        .region(region_provider)
        .load()
        .await
}
//======================================== END AWS

#[derive(Debug)]
pub enum RunType {
    Standard,
    Captioning,
}

#[derive(Debug)]
struct BedrockCall {
    pub body: Blob,
    pub content_type: &'static str,
    pub accept: &'static str,
    pub model_id: String,
}

impl BedrockCall {
    fn new(
        body: &dyn Blobbable,
        content_type: &'static str,
        accept: &'static str,
        model_id: String,
    ) -> BedrockCall {
        let body = body.to_blob();
        BedrockCall {
            body,
            content_type,
            accept,
            model_id,
        }
    }
}

// Eventually this wil need to support every model in ArgModels, but
// this will not necessarily be a 1-to-1 mapping. For example, minor
// version updates to the model will have the same body, but differnet
// values than in ArgModels. Thus, |ArgModels| >= |BedrockCallSum|.
enum ModelOptions {
    Cohere {
        model_id: String,
        body: CohereBody,
    },
    Claude {
        model_id: String,
        body: ClaudeBody,
    },
    Claude3 {
        model_id: String,
        body: ClaudeV3Body,
    },
    Llama2 {
        model_id: String,
        body: Llama2Body,
    },
    Jurrasic2 {
        model_id: String,
        body: Jurrasic2Body,
    },
    TitanText {
        model_id: String,
        body: TitanTextV1Body,
    },
    Mistral7b {
        model_id: String,
        body: Mistral7Body,
    },
}

// Using a sum type to represent all models that can go through here.
// This way if each model needs special processing to make a BedrockCall
// that can be implemented in one place.
fn model_options_to_bedrock_call(bcs: ModelOptions) -> Result<BedrockCall> {
    match bcs {
        ModelOptions::Cohere { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
        ModelOptions::Claude { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
        ModelOptions::Claude3 { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
        ModelOptions::Llama2 { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
        ModelOptions::Jurrasic2 { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
        ModelOptions::TitanText { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
        ModelOptions::Mistral7b { model_id, body } => {
            Ok(BedrockCall::new(&body, "application/json", "*/*", model_id))
        }
    }
}

// Create a `ModelOptions` with sensible defaults for each model.
// This will fail if model_id is not known to q_to_bcs_with_defaults.
fn convert_question_to_model_options(
    question: Option<String>,
    model_id: &str,
    image: Option<&Image>,
) -> Result<ModelOptions, anyhow::Error> {
    // call the function to load model settings:
    let model_defaults = load_model_config()?;

    match model_id {
        "meta.llama2-70b-chat-v1" => {
            let d = model_defaults.llama270b;
            let llama2_body = Llama2Body::new(
                question
                    .ok_or_else(|| anyhow!("There was no question passed to Llama2 70B Chat"))?,
                d.temperature,
                d.p,
                d.max_gen_len,
            );
            Ok(ModelOptions::Llama2 {
                model_id: String::from("meta.llama2-70b-chat-v1"),
                body: llama2_body,
            })
        }
        "cohere.command-text-v14" => {
            let d = model_defaults.cohere_command;
            let cohere_body = CohereBody::new(
                question.ok_or_else(|| {
                    anyhow!("There was no question passed to Cohere Command Text")
                })?,
                d.max_tokens,
                d.temperature,
                d.p,
                d.k,
                d.stop_sequences,
                d.stream,
            );

            Ok(ModelOptions::Cohere {
                model_id: String::from("cohere.command-text-v14"),
                body: cohere_body,
            })
        }
        "ai21.j2-ultra-v1" => {
            let d = model_defaults.jurrasic_2_ultra;
            let jurrasic_body = Jurrasic2Body::new(
                question
                    .ok_or_else(|| anyhow!("There was no question passed to Jurrasic 2 Ultra"))?,
                d.temperature,
                d.top_p,
                d.max_tokens,
                d.stop_sequences,
            );
            Ok(ModelOptions::Jurrasic2 {
                model_id: String::from("ai21.j2-ultra-v1"),
                body: jurrasic_body,
            })
        }
        "anthropic.claude-v2" => {
            // TODO: Move to the messages api from v3
            let d = model_defaults.claude_v2;
            let q = question.ok_or_else(|| anyhow!("There was no question passed to Claude v2"))?;
            let claude_body = ClaudeBody::new(
                format!("\n\nHuman: {}\n\nAssistant:", q).to_string(),
                d.temperature,
                d.p,
                d.k,
                d.max_tokens_to_sample,
                d.stop_sequences,
            );
            Ok(ModelOptions::Claude {
                model_id: String::from("anthropic.claude-v2"),
                body: claude_body,
            })
        }
        "anthropic.claude-3-sonnet-20240229-v1:0" => {
            let claude_image: Option<ClaudeImageSource> = if image.is_some() {
                Some(ClaudeImageSource {
                    image_type: "base64".to_string(),
                    data: image.as_ref().unwrap().base64.clone(),
                    media_type: format!("image/{}", image.as_ref().unwrap().extension),
                })
            } else {
                None
            };
            let d = model_defaults.claude_v3;
            let claudev3_body = ClaudeV3Body::new(
                d.anthropic_version,
                d.max_tokens,
                d.role,
                d.default_content_type,
                question,
                claude_image,
            );
            Ok(ModelOptions::Claude3 {
                model_id: String::from("anthropic.claude-3-sonnet-20240229-v1:0"),
                body: claudev3_body,
            })
        }
        "anthropic.claude-3-haiku-20240307-v1:0" => {
            let claude_image: Option<ClaudeImageSource> = if image.is_some() {
                Some(ClaudeImageSource {
                    image_type: "base64".to_string(),
                    data: image.as_ref().unwrap().base64.clone(),
                    media_type: format!("image/{}", image.as_ref().unwrap().extension),
                })
            } else {
                None
            };
            let d = model_defaults.claude_v3;
            let claudev3_body = ClaudeV3Body::new(
                d.anthropic_version,
                d.max_tokens,
                d.role,
                d.default_content_type,
                question,
                claude_image,
            );
            Ok(ModelOptions::Claude3 {
                model_id: String::from("anthropic.claude-3-haiku-20240307-v1:0"),
                body: claudev3_body,
            })
        }
        "anthropic.claude-v2:1" => {
            let d = model_defaults.claude_v21;
            // TODO: Move to the messages api from v3
            let q =
                question.ok_or_else(|| anyhow!("There was no question passed to Claude v21"))?;
            let claude_body = ClaudeBody::new(
                format!("\n\nHuman: {}\n\nAssistant:", q).to_string(),
                d.temperature,
                d.p,
                d.k,
                d.max_tokens_to_sample,
                d.stop_sequences,
            );
            Ok(ModelOptions::Claude {
                model_id: String::from("anthropic.claude-v2:1"),
                body: claude_body,
            })
        }
        "amazon.titan-text-express-v1" => {
            let d = model_defaults.titan_text_express_v1;
            let titan_body = TitanTextV1Body::new(
                question.ok_or_else(|| {
                    anyhow!("There was no question passed to Titan Text V1 Express")
                })?,
                d.temperature,
                d.top_p,
                d.max_token_count,
                d.stop_sequences,
            );
            Ok(ModelOptions::TitanText {
                model_id: String::from("amazon.titan-text-express-v1"),
                body: titan_body,
            })
        }
        "mistral.mixtral-8x7b-instruct-v0:1" => {
            let d = model_defaults.mixtral_8x7b_instruct;
            let mixtral_body = Mistral7Body::new(
                question.ok_or_else(|| anyhow!("There was no question passed to Mixtral 8x7b"))?,
                d.temperature,
                d.top_p,
                d.top_k,
                d.max_tokens,
                d.stop,
            );
            Ok(ModelOptions::Mistral7b {
                model_id: String::from("mistral.mixtral-8x7b-instruct-v0:1"),
                body: mixtral_body,
            })
        }
        "mistral.mistral-7b-instruct-v0:2" => {
            let d = model_defaults.mistral_7b_instruct;
            let mixtral_body = Mistral7Body::new(
                question.ok_or_else(|| anyhow!("There was no question passed to Mistral 7b"))?,
                d.temperature,
                d.top_p,
                d.top_k,
                d.max_tokens,
                d.stop,
            );
            Ok(ModelOptions::Mistral7b {
                model_id: String::from("mistral.mistral-7b-instruct-v0:2"),
                body: mixtral_body,
            })
        }
        "mistral.mistral-large-2402-v1:0" => {
            let d = model_defaults.mistral_7b_instruct;
            let mixtral_body = Mistral7Body::new(
                question.ok_or_else(|| anyhow!("There was no question passed to Mistral Large"))?,
                d.temperature,
                d.top_p,
                d.top_k,
                d.max_tokens,
                d.stop,
            );
            Ok(ModelOptions::Mistral7b {
                model_id: String::from("mistral.mistral-large-2402-v1:0"),
                body: mixtral_body,
            })
        }
        &_ => todo!(),
    }
}

// Given a question and model_id, create a BedrockCall to this model.
// This will fail if model_id is not known to q_to_bcs_with_defaults.
fn mk_bedrock_call(
    question: &String,
    image: Option<&Image>,
    model_id: &str,
) -> Result<BedrockCall> {
    let bcs = convert_question_to_model_options(Some(question.to_string()), model_id, image)?;
    model_options_to_bedrock_call(bcs)
}

// Given a question and model_id, create and execute a call to bedrock.
// This will fail if model_id is not known to q_to_bcs_with_defaults
pub async fn ask_bedrock(
    question: &String,
    image: Option<&Image>,
    model_id: &str,
    run_type: RunType,
    client: &aws_sdk_bedrockruntime::Client,
    bedrock_client: &aws_sdk_bedrock::Client,
) -> Result<String, anyhow::Error> {
    match run_type {
        RunType::Standard => {
            let bcall = mk_bedrock_call(question, image, model_id)?;
            // check if model supports streaming:
            if check_for_streaming(model_id.to_string(), bedrock_client).await? {
                let response = call_bedrock_stream(client, bcall).await?;
                Ok(response)
            } else {
                // if it does not just call it
                let response = call_bedrock(client, bcall, run_type).await?;
                Ok(response)
            }
        }
        RunType::Captioning => {
            if image.is_some() {
                // TODO: Programmaticall check for multimodality of FMs
                if model_id != "anthropic.claude-3-sonnet-20240229-v1:0"
                    && model_id != "anthropic.claude-3-haiku-20240307-v1:0"
                {
                    eprintln!("🛑SORRY! The model you selected is not able to caption images. Please select either `claude-v3-sonnet` or `claude-v3-haiku`.");
                    std::process::exit(1);
                }
                let bcall = mk_bedrock_call(question, image, model_id)?;
                // because this is captioniong, we dont need streaming
                let caption = call_bedrock(client, bcall, run_type).await?;
                Ok(caption)
            } else {
                Err(anyhow!("No images provided. Captioning aborted."))
            }
        }
    }
    //Ok(())
}

//========================================

fn process_response(
    model_id: &str,
    payload_bytes: &[u8],
    streaming: bool,
) -> Result<String, serde_json::Error> {
    if !streaming {
        match model_id {
            "anthropic.claude-3-sonnet-20240229-v1:0"
            | "anthropic.claude-3-haiku-20240307-v1:0" => {
                serde_json::from_slice::<ClaudeV3Response>(payload_bytes)
                    .map(|res| res.content[0].text.clone())
            }
            "ai21.j2-ultra-v1" => {
                serde_json::from_slice::<Jurrasic2ResponseCompletions>(payload_bytes)
                    .map(|res| res.completions[0].data.text.clone())
            }
            &_ => Err(serde_json::Error::custom("Unknown model ID")),
        }
    } else {
        match model_id {
            "meta.llama2-70b-chat-v1" => {
                serde_json::from_slice::<Llama2Response>(payload_bytes).map(|res| res.generation)
            }
            "cohere.command-text-v14" => {
                serde_json::from_slice::<CohereResponseText>(payload_bytes).map(|res| res.text)
            }
            "anthropic.claude-v2" | "anthropic.claude-v2:1" => {
                serde_json::from_slice::<ClaudeResponse>(payload_bytes).map(|res| res.completion)
            }
            "anthropic.claude-3-sonnet-20240229-v1:0"
            | "anthropic.claude-3-haiku-20240307-v1:0" => {
                // NOTE: ClaudeV3 is complicated and the streamed response is not always the same
                // this means we need to check for specific fields in the response and then return only
                // if we have the type of response set to "text_delta"
                // FIX: I feel like this could be way better
                // FIX: Make it so you check for other message types and to something about it.
                let mut deserializer = serde_json::Deserializer::from_slice(payload_bytes);
                let value = Value::deserialize(&mut deserializer)?;
                if let Value::Object(obj) = value {
                    if let Some(Value::Object(delta)) = obj.get("delta") {
                        if let Some(Value::String(delta_type)) = delta.get("type") {
                            if delta_type == "text_delta" {
                                let text = delta
                                    .get("text")
                                    .and_then(|v| v.as_str().map(ToString::to_string))
                                    .ok_or_else(|| Error::custom("text"))?;
                                return Ok(text);
                            }
                        }
                    }
                }
                Ok(String::from(""))
            }
            "amazon.titan-text-express-v1" => {
                serde_json::from_slice::<TitanTextV1Results>(payload_bytes)
                    .map(|res| res.output_text)
            }
            "mistral.mixtral-8x7b-instruct-v0:1" | "mistral.mistral-7b-instruct-v0:2" | "mistral.mistral-large-2402-v1:0" => {
                serde_json::from_slice::<Mistral7Results>(payload_bytes)
                    .map(|res| res.outputs[0].text.clone())
            }
            &_ => Err(serde_json::Error::custom("Unknown model ID")),
        }
    }
}

// this function is only called if we do not want the streaming result back.
// so far this is here only for models that do not support streaming (ie Jurrasic2Ultra)
async fn call_bedrock(
    bc: &aws_sdk_bedrockruntime::Client,
    c: BedrockCall,
    run_type: RunType,
) -> Result<String, anyhow::Error> {
    let response = bc
        .invoke_model()
        .body(c.body)
        .content_type(c.content_type)
        .accept(c.accept)
        .model_id(&c.model_id)
        .send()
        .await
        .map_err(give_bedrock_hints)?;

    let response_text = process_response(c.model_id.as_str(), response.body.as_ref(), false);
    match response_text {
        Ok(text) => match run_type {
            RunType::Captioning => Ok(text),
            RunType::Standard => {
                println!("{}", text);
                Ok(text)
            }
        },
        Err(e) => Err(anyhow!("Error processing response: {}", e)),
    }
}

/// Add context and advice for specific error variants
fn give_bedrock_hints(err: impl Into<aws_sdk_bedrockruntime::Error>) -> anyhow::Error {
    let err = err.into();
    let context = match &err {
        aws_sdk_bedrockruntime::Error::AccessDeniedException(_err) => {
            Some("hint: If you belive you have enabled this model, note that access MUST be enabled for a specific region!")
        }
        _ => None,
    };
    let mut anyhow_err: anyhow::Error = err.into();
    if let Some(context) = context {
        anyhow_err = anyhow_err.context(context);
    }
    anyhow_err = anyhow_err.context("failed to invoke bedrock");

    anyhow_err
}

async fn call_bedrock_stream(
    bc: &aws_sdk_bedrockruntime::Client,
    c: BedrockCall,
) -> Result<String, anyhow::Error> {
    let mut resp = bc
        .invoke_model_with_response_stream()
        .body(c.body)
        .content_type(c.content_type)
        .accept(c.accept)
        .model_id(&c.model_id)
        .send()
        .await
        .map_err(give_bedrock_hints)?;

    let mut output = String::new();

    while let Some(event) = resp.body.recv().await? {
        match event {
            ResponseStream::Chunk(payload_part) => {
                if let Some(payload_bytes) = payload_part.bytes {
                    let response_text =
                        process_response(c.model_id.as_str(), payload_bytes.as_ref(), true);
                    match response_text {
                        Ok(text) => {
                            output.push_str(&text);
                            print!("{}", &text);
                            io::stdout().flush()?;
                        }
                        Err(e) => eprintln!("Error processing response: {}", e),
                    }
                }
            }
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }
    println!();
    Ok(output)
}
