use std::io;
use std::io::Write;
use std::fs;

use dirs::home_dir;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use anyhow::anyhow;
use anyhow::Result;
use aws_sdk_bedrockruntime::types::InferenceConfiguration;
use bedrust::utils;

use bedrust::configure_aws;
use bedrust::utils::prompt_for_model_selection;

use bedrust::captioner::caption_image;
use bedrust::captioner::list_files_in_path_by_extension;
use bedrust::captioner::write_captions;
use bedrust::captioner::Image;
use bedrust::captioner::OutputFormat;
use bedrust::utils::{check_for_config, initialize_config, print_warning, save_chat_history};
use clap::Parser;

use bedrust::code::code_chat;
use bedrust::constants;
use bedrust::models::converse_stream::call_converse_stream;
use bedrust::models::{check_model_features, ModelFeatures};

// TODO:
// So far I've implemented the converse API for general purpose chat and the code chat.
// What I need to do is:
// - Support Images and captioning - [DONE] ✅
//  - Check if model supports images before attempting to run - [DONE] ✅
// - Store the default inference parameters in the config file - [DONE] ✅
// - Figure out feature support matrix for the Converse API and the models. - [DONE] ✅
// - Remove nom v3.2.1 - [DONE] ✅
// - Remove unwanted commented out code - [DONE] ✅
// - Make sure everything works after ripping out the old code - [DONE] ✅

#[tokio::main]
async fn main() -> Result<()> {
    // parsing arguments
    let arguments = utils::Args::parse();
    // Checking for the `--init` flag and then initializing the configuration
    if arguments.init {
        if check_for_config()? {
            print_warning("****************************************");
            print_warning("WARNING:");
            println!("You are trying to initialize the Bedrust configuration");
            println!("This will overwrite your configuration files in $HOME/.config/bedrust/");
            print!("ARE YOU SURE YOU WANT DO TO THIS? Y/N: ");
            io::stdout().flush()?; // so the answers are typed on the same line as above

            let mut confirmation = String::new();
            io::stdin().read_line(&mut confirmation)?;
            if confirmation.trim().eq_ignore_ascii_case("y") {
                print_warning("I ask AGAIN");
                print!("ARE YOU SURE YOU WANT DO TO THIS? Y/N: ");
                io::stdout().flush()?; // so the answers are typed on the same line as above

                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation)?;

                if confirmation.trim().eq_ignore_ascii_case("y") {
                    println!("----------------------------------------");
                    println!("📜 | Initializing Bedrust configuration.");
                    initialize_config()?;
                }
            }
        } else {
            println!("----------------------------------------");
            println!("📜 | Initializing Bedrust configuration.");
            initialize_config()?;
        }
        print_warning("Bedrust will now exit");
        std::process::exit(0);
    }
    // checking if the configuration files exist
    if !check_for_config()? {
        print_warning("****************************************");
        print_warning("WARNING:");
        println!("Your Bedrust configuration files are not set up correctly.");
        println!("To use Bedrust you need the appropriate `bedrust_config.ron and `model_config.ron` in your $HOME/.config/bedrust/ directory.");
        println!("You can configure the application by running `bedrust --init`");
        print_warning("****************************************");
        print_warning("Bedrust will now exit");
        std::process::exit(1);
    }
    // load bedrust config file
    let bedrust_config = utils::load_bedrust_config()?;

    // configuring the SDK
    let config = configure_aws(String::from("us-east-1"), bedrust_config.aws_profile).await;
    // setup the bedrock-runtime client
    let bedrock_runtime_client = aws_sdk_bedrockruntime::Client::new(&config);
    // setup the bedrock client
    let bedrock_client = aws_sdk_bedrock::Client::new(&config);

    //let question = "Which songs are listed in the youtube video 'evolution of dance'?";
    let model_id = arguments.model_id.or(bedrust_config.default_model);
    let model_id = match model_id {
        Some(model_id) => model_id,
        None => prompt_for_model_selection()?,
    }
    .to_str();

    // === DEFAULT INFERENCE PARAMETERS ===
    let inference_parameters = InferenceConfiguration::builder()
        .max_tokens(bedrust_config.inference_params.max_tokens)
        .top_p(bedrust_config.inference_params.top_p)
        .temperature(bedrust_config.inference_params.temperature)
        .build();

    // if we enabled captioning of images
    if arguments.caption.is_some() {
        match check_model_features(model_id, &bedrock_client, ModelFeatures::Images).await {
            Ok(b) => {
                match b {
                    true => {
                        // FIX: This should be a function
                        println!("----------------------------------------");
                        println!("🖼️ | Image captioner running.");
                        let path = arguments
                            .caption
                            .ok_or_else(|| anyhow!("No path specified"))?;
                        println!("⌛ | Processing images in: {:?}", &path);
                        let files =
                            list_files_in_path_by_extension(path, bedrust_config.supported_images)?;
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
                    }
                    false => {
                        eprintln!("The current model selected does not support Images. Please consider using one that does.")
                    }
                }
            }
            Err(e) => eprintln!("Unable to determine model features: {}", e),
        };
    } else {
        // default run
        utils::hello_header("Bedrust")?;

        // BETA: SOURCE READING
        let mut conversation_history = if arguments.source.is_some() {
            println!("----------------------------------------");
            print_warning("⚠ THIS IS A BETA FEATURE ⚠");
            println!("----------------------------------------");
            println!("💾 | Ooh, it Seems we are talking about code today!");
            println!(
                "💾 | I was given this dir to review: {:?}",
                arguments.source.clone().unwrap().into_os_string()
            );
            println!("----------------------------------------");
            let mut convo = String::new();
            convo.push_str(constants::CODE_CHAT_PROMPT);

            let code =
                code_chat(arguments.source.clone().unwrap(), &bedrock_runtime_client).await?;
            println!("----------------------------------------");
            print_warning("⚠ THIS IS A BETA FEATURE ⚠");

            // Return this conversation
            convo.push_str(code.as_str());
            convo
        } else {
            // We are not looking at code
            // Just return an empty string
            String::new()
        };

        // get user input
        let mut current_file: Option<String> = None;
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
            if question == "/q" {
                println!("Bye!");
                break;
            } else if question == "/c" {
                println!("Clearing current chat history");
                conversation_history.clear();
                current_file = None;
                continue;
            } else if question == "/s" {
                let filename = if let Some(ref file) = current_file {
                    file.to_string()
                } else {
                    match save_chat_history(&conversation_history) {
                        Ok(name) => {
                            current_file = Some(name.clone());
                            name
                        }
                        Err(e) => {
                            eprintln!("Error saving chat history: {}", e);
                            continue;
                        }
                    }
                };
                match fs::write(home_dir().unwrap().join(format!(".config/{}/chats/{}", constants::CONFIG_DIR_NAME, filename)), &conversation_history) {
                    Ok(_) => println!("Chat history saved to: {}", filename),
                    Err(e) => eprintln!("Error saving chat history: {}", e),
                }
                continue;
            } else if question == "/r" {
                match utils::list_chat_histories() {
                    Ok(histories) => {
                        if histories.is_empty() {
                            println!("No chat histories found.");
                            continue;
                        }
                        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select a chat history to recall:")
                            .default(0)
                            .items(&histories[..])
                            .interact()
                            .unwrap();
                        let selected_history = &histories[selection];
                        match utils::load_chat_history(selected_history) {
                            Ok(content) => {
                                conversation_history = content;
                                current_file = Some(selected_history.to_string());
                                println!("Loaded chat history from: {}", selected_history);
                                println!("You can now continue the conversation.");
                            }
                            Err(e) => eprintln!("Error loading chat history: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Error listing chat histories: {}", e),
                }
                continue;
            } else if question.starts_with('/') {
                utils::print_warning("Special command detected: /");
                utils::print_warning("----------------------------------------");
                utils::print_warning("Currently supported chat commands: ");
                utils::print_warning("/c\t \t - Clear current chat history");
                utils::print_warning("/s\t \t - Save chat history");
                utils::print_warning("/r\t \t - Recall and load a chat history");
                utils::print_warning("/q\t \t - Quit");
                continue;
            }
            conversation_history.push_str(question);
            conversation_history.push('\n');

            println!("----------------------------------------");
            println!("☎️  | Calling Model: {}", &model_id);
            println!("----------------------------------------");

            let streamresp = call_converse_stream(
                &bedrock_runtime_client,
                model_id.to_string(),
                &conversation_history.to_string(),
                inference_parameters.clone(),
            )
            .await?;
            conversation_history.push_str(&streamresp);
            conversation_history.push('\n');
        }
    }

    Ok(())
}
