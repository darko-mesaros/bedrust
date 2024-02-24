mod models;

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_types::region::Region;
use core::panic;
use std::{env, io};

use anyhow::Result;

use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::types::ResponseStream;

use models::check_for_streaming;
use models::load_config;

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
    Llama2BCS { model_id: String, body: Llama2Body},
    Jurrasic2BCS { model_id: String, body: Jurrasic2Body}    
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
        BedrockCallSum::Llama2BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Jurrasic2BCS { model_id, body } => {
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
        "anthropic.claude-v2:1" | "anthropic.claude-v2" => {
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

//######################################## COHERE

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
    pub fn new(prompt: String, max_tokens: i32, temperature: f32, p: f32, k: i32, stop_sequences: Vec<String>, stream: bool) -> CohereBody {
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

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize)]
pub struct CohereResponseGenerations {
    generations: Vec<CohereResponseText>,
}
#[derive(serde::Deserialize, Debug)]
pub struct CohereResponseText {
    text: String,
}
//######################################## END COHERE

//######################################## CLAUDE
#[derive(serde::Serialize, Debug)]
pub struct ClaudeBody {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens_to_sample: i32,
    pub stop_sequences: Vec<String>,
}

impl ClaudeBody {
    pub fn new(prompt: String, temperature: f32, top_p: f32, top_k: i32, max_tokens_to_sample: i32, stop_sequences: Vec<String>) -> ClaudeBody {
        ClaudeBody {
            prompt,
            temperature,
            top_p,
            top_k,
            max_tokens_to_sample,
            stop_sequences,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ClaudeResponse {
    completion: String,
}
//######################################## END CLAUDE
//
//######################################## LLAMA2
#[derive(serde::Serialize, Debug)]
pub struct Llama2Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_gen_len: i32,
}

impl Llama2Body {
    pub fn new(prompt: String, temperature: f32, top_p: f32, max_gen_len: i32) -> Llama2Body {
        Llama2Body {
            prompt,
            temperature,
            top_p,
            max_gen_len,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Llama2Response {
    generation: String,
}
//######################################## END CLAUDE
//######################################## START JURRASIC
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
    pub fn new(prompt: String, temperature: f32, top_p: f32, max_tokens: i32, stop_sequences: Vec<String>) -> Jurrasic2Body {
        Jurrasic2Body {
            prompt,
            temperature,
            top_p,
            max_tokens,
            stop_sequences
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Jurrasic2ResponseCompletions {
   completions: Vec<Jurrasic2ResponseData>,
}
#[derive(serde::Deserialize, Debug)]
pub struct Jurrasic2ResponseData {
   data: Jurrasic2ResponseText,
}

#[derive(serde::Deserialize, Debug)]
pub struct Jurrasic2ResponseText {
   text: String,
}
//######################################## END JURRASIC
//========================================


// this function is only called if we do not want the streaming result back.
// so far this is here only for legacy reasons
async fn call_bedrock(bc: aws_sdk_bedrockruntime::Client, c: BedrockCall) -> Result<()>{

    let response = bc.invoke_model()
    .body(c.body)
    .content_type(c.content_type)
    .accept(c.accept)
    .model_id(&c.model_id)
    .send()
    .await?;


    let response_body = response
        .body
        .into_inner();

    match c.model_id.as_str() {
        "meta.llama2-70b-chat-v1" => {
            if let Ok(response_body) = serde_json::from_slice::<Llama2Response>(response_body.as_ref()) {
                println!("{}", response_body.generation);
            }
        },
        "cohere.command-text-v14" => {
            if let Ok(response_body) = serde_json::from_slice::<CohereResponseText>(response_body.as_ref()) { 
                println!("{}", response_body.text);
            }
        },
        "anthropic.claude-v2" | "anthropic.claude-v2:1" => {
            if let Ok(response_body) = serde_json::from_slice::<ClaudeResponse>(response_body.as_ref()) {
                println!("{}", response_body.completion);
           }
        },
        "ai21.j2-ultra-v1" => {
            if let Ok(response_body) = serde_json::from_slice::<Jurrasic2ResponseCompletions>(response_body.as_ref()) {
                println!("{}", response_body.completions[0].data.text);
            }
        },
        &_ => todo!()
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
                    match c.model_id.as_str()  {
                        "meta.llama2-70b-chat-v1" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<Llama2Response>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.generation);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.generation;
                            }
                        },
                        "cohere.command-text-v14" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<CohereResponseText>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.text);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.text;
                            }
                        },
                        "anthropic.claude-v2" | "anthropic.claude-v2:1" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<ClaudeResponse>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.completion);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.completion;
                            }
                        },
                        "ai21.j2-ultra-v1" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<Jurrasic2ResponseText>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.text);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.text;
                            }
                        },
                        &_ => todo!()
                    }
                }
            },
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }
    Ok(())
}
