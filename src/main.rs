mod utils;

use anyhow::Result;

use aws_sdk_bedrockruntime::Client;
use bedrust::CohereBody;
use bedrust::Llama2Body;
use bedrust::configure_aws;
use bedrust::call_bedrock_stream;
use bedrust::ClaudeBody;
use bedrust::BedrockCall;

#[tokio::main]
async fn main() -> Result<()>{
    // configuring the SDK
    let config =  configure_aws(String::from("us-west-2")).await;

    // setup the bedrock client
    let bedrock_client = Client::new(&config);

    // VARIABLES
    let question = "List all of the memes in the Weezer song 'pork and beans'";
    let model_id = "anthropic.claude-v2"
    let bedrock_call = q_to_bcs_with_defaults(&question, &model_id)
    utils::hello_header("Welcome to Bedrust");

    println!("----------------------------------------");
    println!("Calling Model: {}", &model_id);
    println!("Question being asked: {}", &question);
    println!("----------------------------------------");
    call_bedrock_stream(bedrock_client, bedrock_call).await?;
    Ok(())
}
