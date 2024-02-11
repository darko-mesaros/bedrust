mod utils;

use anyhow::Result;

use aws_sdk_bedrockruntime::Client;

use bedrust::configure_aws;
use bedrust::call_bedrock_stream;
use bedrust::ask_bedrock;



#[tokio::main]
async fn main() -> Result<()>{
    // configuring the SDK
    let config =  configure_aws(String::from("us-west-2")).await;

    // setup the bedrock client
    let bedrock_client = Client::new(&config);

    // VARIABLES
    let question = "Which songs are listed in the classic dancing guy youtube video 'evolution of dance'";
    let model_id = "anthropic.claude-v2";

    utils::hello_header("Welcome to Bedrust");

    println!("----------------------------------------");
    println!("Calling Model: {}", &model_id);
    println!("Question being asked: {}", &question);
    println!("----------------------------------------");
    ask_bedrock(question.to_string(), model_id, bedrock_client).await?;

    Ok(())
}
