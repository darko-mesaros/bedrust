use anyhow::Result;

use aws_sdk_bedrockruntime::Client;
use bedrust::configure_aws;
use bedrust::convert_json;
use bedrust::call_bedrock;
use bedrust::CohereBody;
use bedrust::BedrockCall;

#[tokio::main]
async fn main() -> Result<()>{
    // configuring the SDK
    let config =  configure_aws(String::from("us-west-2")).await;

    // setup the bedrock client
    let bedrock_client = Client::new(&config);

    // create a CohereBody
    let cohere_body = CohereBody::new(
            // prompt
            String::from("When was Rust the programming language invented?"), 
            // max_tokens
            String::from("500"), 
            // temperature
            String::from("1"), 
            // p
            String::from("1"),
            // k
            String::from("0")
            );

    let cohere_call = BedrockCall::new(
        // body blob
        cohere_body.convert_to_blob(),
        // content-type
        String::from("application/json"), 
        // accept
        String::from("*/*"), 
        // model-id
        String::from("cohere.command-text-v14"));


    let response_string = call_bedrock(bedrock_client, cohere_call).await;

    let result = convert_json(&response_string.unwrap())?;
    println!("{}", result);

    Ok(())

}
