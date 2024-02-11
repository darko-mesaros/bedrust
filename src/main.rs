mod utils;

use anyhow::Result;
use clap::Parser;
use aws_sdk_bedrockruntime::Client;

use bedrust::CohereBody;
use bedrust::Llama2Body;
use bedrust::configure_aws;
use bedrust::call_bedrock_stream;
use bedrust::ClaudeBody;
use bedrust::BedrockCall;

#[tokio::main]
async fn main() -> Result<()>{

    // parse arguments
    let arguments = utils::Args::parse();

    // configuring the SDK
    let config =  configure_aws(String::from("us-west-2")).await;

    // setup the bedrock client
    let bedrock_client = Client::new(&config);

    // VARIABLES
    let question = "Who is Alan Ford, a comic book character?";

    // get model from argument
    let model_id = arguments.model_id.to_str();

    // run start
    let bedrock_call: BedrockCall = match model_id {
        "meta.llama2-70b-chat-v1" => {
            let llama2_body = Llama2Body::new(
                // prompt
                question.to_string(),
                // temperature
                1.0,
                // p
                0.1,
                // max_gen_len
                1024
                );
            BedrockCall::new(
                llama2_body.convert_to_blob()?,
                String::from("application/json"), 
                // accept
                String::from("*/*"), 
                // model-id
                model_id.to_string(),
            )
        },
        "cohere.command-text-v14" => {
            let cohere_body = CohereBody::new(
                // prompt
                question.to_string(),
                // max tokens
                500,
                // temperature
                1.0,
                // p
                0.1,
                // k
                1,
                // stop sequences
                Vec::new(),
                // stream
                true,
                );

            BedrockCall::new(
                cohere_body.convert_to_blob()?,
                String::from("application/json"), 
                // accept
                String::from("*/*"), 
                // model-id
                model_id.to_string(),
            )
        },
        "anthropic.claude-v2" => {
            let claude_body = ClaudeBody::new(
                // prompt
                format!("\n\nHuman: {}\n\nAssistant:", question).to_string(),
                // temp
                1.0,
                // top p
                1.0,
                // top k
                250,
                // max tokens to sample
                500,
                // stop sequences
                Vec::new(),
            );

            BedrockCall::new(
                claude_body.convert_to_blob()?,
                String::from("application/json"), 
                // accept
                String::from("*/*"), 
                // model-id
                model_id.to_string(),
            )

        },
        &_ => todo!()
    };

    utils::hello_header("Welcome to Bedrust");

    println!("----------------------------------------");
    println!("Calling Model: {}", &model_id);
    println!("Question being asked: {}", &question);
    println!("----------------------------------------");
    call_bedrock_stream(bedrock_client, bedrock_call).await?;
    Ok(())
}
