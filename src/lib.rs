pub mod models;

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_types::region::Region;
use serde::Deserialize;
use serde::ser::Error;
use serde_json::Value;
use core::panic;
use std::{env, io};

use anyhow::Result;

use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::types::ResponseStream;

use models::check_for_streaming;
use models::load_config;

use models::cohere::{CohereBody, CohereResponseText};
use models::claude::{ClaudeBody, ClaudeResponse};
use models::claudev3::{ClaudeV3Body};
use models::llama2::{Llama2Body, Llama2Response};
use models::jurrasic2::{Jurrasic2Body, Jurrasic2ResponseCompletions};
use models::titan::{TitanTextV1Body, TitanTextV1Results};
use models::mistral::{Mistral7Body, Mistral7Results};

use std::io::Write;

//======================================== AWS
pub async fn configure_aws(s: String) -> aws_config::SdkConfig {
    let provider = RegionProviderChain::first_try(env::var("AWS_DEFAULT_REGION").ok().map(Region::new))
        .or_default_provider()
        .or_else(Region::new(s));

    aws_config::defaults(BehaviorVersion::latest())
        .region(provider)
        .load()
        .await

}
//======================================== END AWS

#[derive(Debug)]
struct BedrockCall {
    pub body: Blob,
    pub content_type: String,
    pub accept: String,
    pub model_id: String,
}

impl BedrockCall {
    fn new(body: Blob, content_type: String, accept: String, model_id: String ) -> BedrockCall {
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
enum BedrockCallSum {
    CohereBCS { model_id: String, body: CohereBody},
    ClaudeBCS { model_id: String, body: ClaudeBody},
    Claude3BCS { model_id: String, body: ClaudeV3Body},
    Llama2BCS { model_id: String, body: Llama2Body},
    Jurrasic2BCS { model_id: String, body: Jurrasic2Body},
    TitanTextBCS { model_id: String, body: TitanTextV1Body},
    Mistral7bBCS { model_id: String, body: Mistral7Body}
}

// Using a sum type to represent all models that can go through here.
// This way if each model needs special processing to make a BedrockCall
// that can be implemented in one place.
fn bcs_to_bedrock_call(bcs: BedrockCallSum) ->  Result<BedrockCall> {
    match bcs {
        BedrockCallSum::CohereBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::ClaudeBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Claude3BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Llama2BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Jurrasic2BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::TitanTextBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Mistral7bBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
	
    }
}

// Create a BedrockCallSum with sensible defaults for each model.
// This will fail if model_id is not known to q_to_bcs_with_defaults.
fn q_to_bcs_with_defaults(question: String, model_id: &str) -> Result<BedrockCallSum> {
    // call the function to load model settings:
    // TODO: do not hardcode the name and path of the config file
    let model_defaults = load_config(String::from("model_config.ron"))?;

    match model_id {
        "meta.llama2-70b-chat-v1" => {
            let d = model_defaults.llama270b;
            let llama2_body = Llama2Body::new(
                question.to_string(),
                d.temperature,
                d.p,
                d.max_gen_len
                );
	    Ok(BedrockCallSum::Llama2BCS{model_id: String::from("meta.llama2-70b-chat-v1"), body: llama2_body})
	    
        },
        "cohere.command-text-v14" => {
            let d = model_defaults.cohere_command;
            let cohere_body = CohereBody::new(
                question.to_string(),
                d.max_tokens,
                d.temperature,
                d.p,
                d.k,
                d.stop_sequences,
                d.stream,
                );

	    Ok(BedrockCallSum::CohereBCS{model_id: String::from("cohere.command-text-v14"), body: cohere_body})
        },
        "ai21.j2-ultra-v1" => {
            let d = model_defaults.jurrasic_2_ultra;
            let jurrasic_body = Jurrasic2Body::new(
                question.to_string(),
                d.temperature, 
                d.top_p, 
                d.max_tokens, 
                d.stop_sequences,
            );
	    Ok(BedrockCallSum::Jurrasic2BCS{model_id: String::from("ai21.j2-ultra-v1"), body: jurrasic_body})
        },
        "anthropic.claude-v2" => {
            let d = model_defaults.claude_v2;
            let claude_body = ClaudeBody::new(
                format!("\n\nHuman: {}\n\nAssistant:", question).to_string(),
                d.temperature, 
                d.p, 
                d.k, 
                d.max_tokens_to_sample, 
                d.stop_sequences, 
            );
	    Ok(BedrockCallSum::ClaudeBCS{model_id: String::from("anthropic.claude-v2"), body: claude_body})
        },
        "anthropic.claude-3-sonnet-20240229-v1:0" => {
            let d = model_defaults.claude_v3;
            let claudev3_body = ClaudeV3Body::new(
                d.anthropic_version,
                d.max_tokens,
                d.role,
                //NOTE: hardcoded content_type for now (until chat is implemented)
                String::from("text"),
                question.to_string()
            );
	    Ok(BedrockCallSum::Claude3BCS{model_id: String::from("anthropic.claude-3-sonnet-20240229-v1:0"), body: claudev3_body})
        },
        "anthropic.claude-3-haiku-20240307-v1:0" => {
            let d = model_defaults.claude_v3;
            let claudev3_body = ClaudeV3Body::new(
                d.anthropic_version,
                d.max_tokens,
                d.role,
                //NOTE: hardcoded content_type for now (until chat is implemented)
                String::from("text"),
                question.to_string()
            );
	    Ok(BedrockCallSum::Claude3BCS{model_id: String::from("anthropic.claude-3-haiku-20240307-v1:0"), body: claudev3_body})
        },
        "anthropic.claude-v2:1" => {
            let d = model_defaults.claude_v21;
            let claude_body = ClaudeBody::new(
                format!("\n\nHuman: {}\n\nAssistant:", question).to_string(),
                d.temperature, 
                d.p, 
                d.k, 
                d.max_tokens_to_sample, 
                d.stop_sequences, 
            );
	    Ok(BedrockCallSum::ClaudeBCS{model_id: String::from("anthropic.claude-v2:1"), body: claude_body})
        },
        "amazon.titan-text-express-v1" => {
            let d = model_defaults.titan_text_express_v1;
            let titan_body = TitanTextV1Body::new(
                question.to_string(),
                d.temperature, 
                d.top_p, 
                d.max_token_count,
                d.stop_sequences, 
            );
	    Ok(BedrockCallSum::TitanTextBCS{model_id: String::from("amazon.titan-text-express-v1"), body: titan_body})
        },
        "mistral.mixtral-8x7b-instruct-v0:1" => {
            let d = model_defaults.mixtral_8x7b_instruct;
            let mixtral_body = Mistral7Body::new(
                question.to_string(),
                d.temperature, 
                d.top_p, 
                d.top_k, 
                d.max_tokens,
                d.stop, 
            );
	    Ok(BedrockCallSum::Mistral7bBCS{model_id: String::from("mistral.mixtral-8x7b-instruct-v0:1"), body: mixtral_body})
        },
        "mistral.mistral-7b-instruct-v0:2" => {
            let d = model_defaults.mistral_7b_instruct;
            let mixtral_body = Mistral7Body::new(
                question.to_string(),
                d.temperature, 
                d.top_p, 
                d.top_k, 
                d.max_tokens,
                d.stop, 
            );
	    Ok(BedrockCallSum::Mistral7bBCS{model_id: String::from("mistral.mistral-7b-instruct-v0:2"), body: mixtral_body})
        },
	&_ => todo!()
    }
}

// Given a question and model_id, create a BedrockCall to this model.
// This will fail if model_id is not known to q_to_bcs_with_defaults.
fn mk_bedrock_call(question: String, model_id: &str) -> Result<BedrockCall> {
    let bcs = q_to_bcs_with_defaults(question.to_string(), model_id)?;
    bcs_to_bedrock_call(bcs)
}

// Given a question and model_id, create and execute a call to bedrock.
// This will fail if model_id is not known to q_to_bcs_with_defaults
pub async fn ask_bedrock(question: String, model_id: &str, client: aws_sdk_bedrockruntime::Client, bedrock_client: aws_sdk_bedrock::Client ) -> Result<()>{ 

    let bcall = mk_bedrock_call(question, model_id)?;
    // check if model supports streaming:
    if check_for_streaming(model_id.to_string(), bedrock_client).await? {
        call_bedrock_stream(client, bcall).await?;
    } else {
        // if it does not just call it
        call_bedrock(client, bcall).await?;
    }
    Ok(())
}

//========================================

fn process_response(model_id: &str, payload_bytes: &[u8]) -> Result<String, serde_json::Error> {
    match model_id {
        "meta.llama2-70b-chat-v1" => serde_json::from_slice::<Llama2Response>(payload_bytes)
            .map(|res| res.generation),
        "cohere.command-text-v14" => serde_json::from_slice::<CohereResponseText>(payload_bytes)
            .map(|res| res.text),
        "anthropic.claude-v2" | "anthropic.claude-v2:1" => serde_json::from_slice::<ClaudeResponse>(payload_bytes)
            .map(|res| res.completion),
        "anthropic.claude-3-sonnet-20240229-v1:0" | "anthropic.claude-3-haiku-20240307-v1:0" => {
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
                            let text = delta.get("text").and_then(|v|v.as_str().map(ToString::to_string)).ok_or_else(||Error::custom("text"))?;
                            return Ok(text);
                        }
                    }
                }
            }
            Ok(String::from(""))
           },
        "ai21.j2-ultra-v1" => serde_json::from_slice::<Jurrasic2ResponseCompletions>(payload_bytes)
            .map(|res| res.completions[0].data.text.clone()),
        "amazon.titan-text-express-v1" => serde_json::from_slice::<TitanTextV1Results>(payload_bytes)
            .map(|res| res.output_text),
        "mistral.mixtral-8x7b-instruct-v0:1" | "mistral.mistral-7b-instruct-v0:2" => serde_json::from_slice::<Mistral7Results>(payload_bytes)
            .map(|res| res.outputs[0].text.clone()),
        &_ => Err(serde_json::Error::custom("Unknown model ID")),
    }
}

// this function is only called if we do not want the streaming result back.
// so far this is here only for models that do not support streaming (ie Jurrasic2Ultra)
async fn call_bedrock(bc: aws_sdk_bedrockruntime::Client, c: BedrockCall) -> Result<()>{

    let response = bc.invoke_model()
    .body(c.body)
    .content_type(c.content_type)
    .accept(c.accept)
    .model_id(&c.model_id)
    .send()
    .await?;

    let response_text = process_response(c.model_id.as_str(), response.body.as_ref());
    match response_text {
        Ok(text) => {
            print!("{}", text);
            io::stdout().flush().unwrap();
        },
        Err(e) => eprintln!("Error processing response: {}", e),
    }

    Ok(())

}

async fn call_bedrock_stream(bc: aws_sdk_bedrockruntime::Client, c: BedrockCall) -> Result<()>{

    let mut resp =  bc.invoke_model_with_response_stream()
        .body(c.body)
        .content_type(c.content_type)
        .accept(c.accept)
        .model_id(&c.model_id)
        .send()
        .await?;

    let mut output = String::new();

    while let Some(event) = resp.body.recv().await? {
        match event {
            ResponseStream::Chunk(payload_part) => {
                if let Some(payload_bytes) = payload_part.bytes {
                    let response_text = process_response(c.model_id.as_str(), payload_bytes.as_ref());
                    match response_text {
                        Ok(text) => {
                            print!("{}", text);
                            io::stdout().flush().unwrap();
                            output += &text;
                        },
                        Err(e) => eprintln!("Error processing response: {}", e),
                    }
                }
            },
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }
    Ok(())
}
