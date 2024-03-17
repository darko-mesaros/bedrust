// TODO: 
// CAPTIONER: Implement writing to JSON after captioning
// - This would require to change the ask_bedrock chain
// - That instead of printing to screen, it should print to JSON
// - Rather it should be writtent back to the Image struct and then
//   to JSON.
// - Meaning we need to tell the ask_bedrock chain that this is a 
//   different type of call, and prevent it from printing
// CAPTIONER: Impelement some nice printouts while the caption is 
// happening.
mod utils;
use std::io;

use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;

use bedrust::ask_bedrock;
use bedrust::configure_aws;

use bedrust::captioner::Image;
use bedrust::captioner::list_files_in_path_by_extension;
use bedrust::captioner::caption_image;

#[tokio::main]
async fn main() -> Result<()> {
    // parsing arguments
    let arguments = utils::Args::parse();
    // load bedrust config file
    let bedrust_config = utils::load_bedrust_config(String::from("bedrust_config.ron"))?;
    // configuring the SDK
    let config = configure_aws(String::from("us-west-2")).await;
    // setup the bedrock-runtime client
    let bedrock_runtime_client = aws_sdk_bedrockruntime::Client::new(&config);
    // setup the bedrock client
    let bedrock_client = aws_sdk_bedrock::Client::new(&config);

    //let question = "Which songs are listed in the youtube video 'evolution of dance'?";
    let model_id = arguments.model_id.to_str();

    // if we enabled captioning of images
    if arguments.caption.is_some(){
        let path = arguments.caption.ok_or_else(||anyhow!("No path specified"))?;
        let files = list_files_in_path_by_extension(path, bedrust_config.supported_images)?;
        let mut images: Vec<Image> = Vec::new();
        for file in &files {
            images.push(Image::new(file)?);
        }
        println!("Processing the following images: {:#?}", &files);
        caption_image(images, model_id, &bedrust_config.caption_prompt, &bedrock_runtime_client, &bedrock_client).await?;

    } else {
        // default run
        utils::hello_header("Bedrust")?;
        // get user input
        let mut question = String::new();
        println!("----------------------------------------");
        println!("What would you like to know today?");
        println!("Human: ");
        io::stdin().read_line(&mut question).unwrap();

        println!("----------------------------------------");
        println!("Calling Model: {}", &model_id);
        println!("----------------------------------------");
        ask_bedrock(
            &question.to_string(),
            None,
            model_id,
            &bedrock_runtime_client,
            &bedrock_client,
        )
        .await?;
    }

    Ok(())
}
