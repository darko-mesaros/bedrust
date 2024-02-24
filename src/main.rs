pub mod utils;
pub mod models;

use std::io;

use clap::Parser;
use anyhow::Result;

use bedrust::configure_aws;
use bedrust::ask_bedrock;

#[tokio::main]
async fn main() -> Result<()>{
    // parsing arguments
    let arguments  = utils::Args::parse();
    // configuring the SDK
    let config =  configure_aws(String::from("us-west-2")).await;
    // setup the bedrock-runtime client
    let bedrock_runtime_client = aws_sdk_bedrockruntime::Client::new(&config);
    // setup the bedrock client
    let bedrock_client = aws_sdk_bedrock::Client::new(&config);

    //let question = "Which songs are listed in the youtube video 'evolution of dance'?";
    let model_id = arguments.model_id.to_str();

    utils::hello_header("Welcome to Bedrust");
    // get user input
    let mut question = String::new();
    println!("----------------------------------------");
    println!("What would you like to know today?");
    println!("Human: ");
    io::stdin().read_line(&mut question).unwrap();

    println!("----------------------------------------");
    println!("Calling Model: {}", &model_id);
    println!("----------------------------------------");
    ask_bedrock(question.to_string(), model_id, bedrock_runtime_client, bedrock_client).await?;

    Ok(())
}
