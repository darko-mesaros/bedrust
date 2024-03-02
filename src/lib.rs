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
    Jurrasic2BCS { model_id: String, body: Jurrasic2Body},
    TitanTextBCS { model_id: String, body: TitanTextV1Body},
    Mixtral8x7bBCS { model_id: String, body: Mixtral8x7Body},
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
        BedrockCallSum::Llama2BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Jurrasic2BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::TitanTextBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".to_string(), "*/*".to_string(), model_id))
        }
        BedrockCallSum::Mixtral8x7bBCS { model_id, body } => {
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
            let mixtral_body = Mixtral8x7Body::new(
                question.to_string(),
                d.temperature, 
                d.top_p, 
                d.top_k, 
                d.max_tokens,
                d.stop, 
            );
	    Ok(BedrockCallSum::Mixtral8x7bBCS{model_id: String::from("mistral.mixtral-8x7b-instruct-v0:1"), body: mixtral_body})
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
//######################################## START TITAN
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TitanTextV1Body {
    pub input_text: String,
    pub text_generation_config: TitanTextV1textGenerationConfig
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TitanTextV1textGenerationConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub max_token_count: i32,
    pub stop_sequences: Vec<String>,
}

impl TitanTextV1Body {
    pub fn new(input_text: String, temperature: f32, top_p: f32, max_token_count: i32, stop_sequences: Vec<String>) -> TitanTextV1Body {
        let text_gen_config = TitanTextV1textGenerationConfig {
            temperature,
            top_p,
            max_token_count,
            stop_sequences
        };
        TitanTextV1Body {
            input_text,
            text_generation_config: text_gen_config
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct TitanTextV1Response {
   results: Vec<TitanTextV1Results>
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TitanTextV1Results {
   output_text: String,
}
//######################################## END TITAN
//######################################## START MIXTRAL
#[derive(serde::Serialize, Debug)]
pub struct Mixtral8x7Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub stop: Vec<String>,
}

impl Mixtral8x7Body {
    pub fn new(prompt: String, temperature: f32, top_p: f32, top_k: i32, max_tokens: i32, stop: Vec<String>) -> Mixtral8x7Body {
        Mixtral8x7Body {
            prompt,
            temperature,
            top_p,
            top_k,
            max_tokens,
            stop,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Mixtral8x7Results {
   outputs: Vec<Mixtral8x7Outputs>
}

#[derive(serde::Deserialize, Debug)]
pub struct Mixtral8x7Outputs {
   text: String,
}
//######################################## END MIXTRAL
//######################################## START MISTRAL
#[derive(serde::Serialize, Debug)]
pub struct Mistral7Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub stop: Vec<String>,
}

impl Mistral7Body {
    pub fn new(prompt: String, temperature: f32, top_p: f32, top_k: i32, max_tokens: i32, stop: Vec<String>) -> Mistral7Body {
        Mistral7Body {
            prompt,
            temperature,
            top_p,
            top_k,
            max_tokens,
            stop,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Mistral7Results {
   outputs: Vec<Mistral7Outputs>
}

#[derive(serde::Deserialize, Debug)]
pub struct Mistral7Outputs {
   text: String,
}
//######################################## END MIXTRAL
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
        "amazon.titan-text-express-v1" => {
            if let Ok(response_body) = serde_json::from_slice::<TitanTextV1Results>(response_body.as_ref()) {
                println!("{}", response_body.output_text);
            }
        },
        "mistral.mixtral-8x7b-instruct-v0:1" => {
            if let Ok(response_body) = serde_json::from_slice::<Mixtral8x7Outputs>(response_body.as_ref()) {
                println!("{}", response_body.text);
            }
        },
        "mistral.mistral-7b-instruct-v0:2" => {
            if let Ok(response_body) = serde_json::from_slice::<Mistral7Outputs>(response_body.as_ref()) {
                println!("{}", response_body.text);
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
                        "amazon.titan-text-express-v1" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<TitanTextV1Results>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.output_text);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.output_text;
                            }
                        },
                        "mistral.mixtral-8x7b-instruct-v0:1" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<Mixtral8x7Results>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.outputs[0].text);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.outputs[0].text;
                            }
                        },
                        "mistral.mistral-7b-instruct-v0:2" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<Mistral7Results>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.outputs[0].text);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.outputs[0].text;
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
