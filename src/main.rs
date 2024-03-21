mod utils;
use std::io;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;

use bedrust::ask_bedrock;
use bedrust::configure_aws;
use bedrust::RunType;

use bedrust::captioner::caption_image;
use bedrust::captioner::list_files_in_path_by_extension;
use bedrust::captioner::write_captions;
use bedrust::captioner::Image;
use bedrust::captioner::OutputFormat;

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
    if arguments.caption.is_some() {
        // FIX: This should be a function
        println!("----------------------------------------");
        println!("🖼️ | Image captioner running.");
        let path = arguments
            .caption
            .ok_or_else(|| anyhow!("No path specified"))?;
        println!("⌛ | Processing images in: {:?}", &path);
        let files = list_files_in_path_by_extension(path, bedrust_config.supported_images)?;
        println!("🔎 | Found {:?} images in path.", &files.len());

        let mut images: Vec<Image> = Vec::new();
        for file in &files {
            images.push(Image::new(file)?);
        }

        caption_image(
            &mut images,
            model_id,
            &bedrust_config.caption_prompt,
            &bedrock_runtime_client,
            &bedrock_client,
        )
        .await?;

        // NOTE: This is parsing the `-x` argument and then writing or not, an XML file
        // Thanks StellyUK <3

        // FIX: This whole if else statement does not look nice.
        // i feel it can be better. As doing the whole logic
        // behind an expression seems ... weird
        let outfile = if arguments.xml {
            let outfile = "captions.xml";
            write_captions(images, OutputFormat::Xml, outfile)?;
            outfile
        } else {
            let outfile = "captions.json";
            write_captions(images, OutputFormat::Json, outfile)?;
            outfile
        };
        println!(
            "✅ | Captioning complete, find the generated captions in `{}`",
            outfile
        );
        println!("----------------------------------------");
    } else {
        // default run
        utils::hello_header("Bedrust")?;
        // STORE HISTORY:
        let mut conversation_history = String::new();
        // get user input
        loop {
            println!("----------------------------------------");
            println!("🤖 | What would you like to know today?");
            print!("😎 | Human: ");
            io::stdout().flush()?; // so the question is typed on the same line as above
            
            let mut question = String::new();
            io::stdin().read_line(&mut question)?;

            let question = question.trim();
            if question.is_empty() {
                println!("Please enter a question.");
                continue;
            }
            if question == "/q"{
                println!("Bye!");
                break;
            } else if question.starts_with('/') {
                utils::print_warning("Special command detected: /");
                utils::print_warning("----------------------------------------");
                utils::print_warning("Currently supported chat commands: ");
                utils::print_warning("/q\t \t - Quit");
                continue;
            } 
            conversation_history.push_str(question);
            conversation_history.push('\n');

            println!("----------------------------------------");
            println!("☎️  | Calling Model: {}", &model_id);
            println!("----------------------------------------");
            let response = ask_bedrock(
                //&question.to_string(),
                &conversation_history.to_string(),
                None,
                model_id,
                RunType::Standard,
                &bedrock_runtime_client,
                &bedrock_client,
            )
            .await?;
            conversation_history.push_str(&response);
            conversation_history.push('\n');

        }
    }

    Ok(())
}
