use std::io;
use std::io::Write;

use anyhow::Result;
use aws_sdk_bedrockruntime::types::ContentBlock;
use aws_sdk_bedrockruntime::types::ConversationRole;
use aws_sdk_bedrockruntime::types::InferenceConfiguration;
use aws_sdk_bedrockruntime::types::Message;
use bedrust::config;
use bedrust::utils;
use colored::*;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use bedrust::configure_aws;
use bedrust::utils::prompt_for_model_selection;

use bedrust::captioner::caption_process;
use bedrust::chat::{
    list_chat_histories, load_chat_history, print_conversation_history, save_chat_history,
    ConversationHistory,
};
use bedrust::config::check_for_config;
use bedrust::utils::print_warning;
use clap::Parser;

use bedrust::code::code_chat_process;
use bedrust::models::converse_stream::call_converse_stream;

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
//
// IDEA: Long term memory (facts about a user)
// - Medium term (stuff we talked about in the past)
// - Short term ( current conversation )
// Storing all the facts about you

#[tokio::main]
async fn main() -> Result<()> {
    // parsing arguments
    let arguments = utils::Args::parse();
    // Checking for the `--init` flag and then initializing the configuration
    if arguments.init {
        config::prompt_init_config()?;
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
    let bedrust_config = config::load_bedrust_config()?;

    // configuring the SDK
    let config = configure_aws(String::from("us-east-1"), &bedrust_config.aws_profile).await;
    // setup the bedrock-runtime client
    let bedrock_runtime_client = aws_sdk_bedrockruntime::Client::new(&config);
    // setup the bedrock client
    let bedrock_client = aws_sdk_bedrock::Client::new(&config);

    //let question = "Which songs are listed in the youtube video 'evolution of dance'?";
    // let model_id = arguments.model_id.or(bedrust_config.default_model);
    let model_id = arguments
        .model_id
        .or_else(|| bedrust_config.get_default_model());
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

    // === SYSTEM PROMPT ===
    let system_prompt = bedrust_config
        .system_prompt
        .as_deref()
        .unwrap_or("You are a helpful assistant");

    //  === CAPTIONING RUN ===
    if arguments.caption.is_some() {
        caption_process(
            model_id,
            &bedrock_client,
            &bedrock_runtime_client,
            arguments.caption,
            &bedrust_config,
            arguments.xml,
        )
        .await?;
    } else {
        // default run
        utils::hello_header("Bedrust")?;

        let mut message_count = 0;
        let mut conversation_history = ConversationHistory::new(None, None, None, None);
        let mut current_file: Option<String> = None;

        //  === BETA: SOURCE CODE CHAT ===
        let code: Option<String> = match arguments.source {
            Some(ref source_path) => {
                Some(code_chat_process(source_path.to_path_buf(), &bedrock_runtime_client).await?)
            }
            None => None,
        };
        // get user input
        loop {
            println!("----------------------------------------");
            println!("🤖 | What would you like to know today?");
            print!("😎 | Human: ");
            io::stdout().flush()?; // so the question is typed on the same line as above

            let mut question = String::new();
            io::stdin().read_line(&mut question)?;
            message_count += 1;

            let question = question.trim();
            if question.is_empty() {
                println!("Please enter a question.");
                continue;
            }
            if question == "/q" {
                println!("Bye!");
                break;
            } else if question == "/h" {
                conversation_history.save_as_html()?;
                continue;
            } else if question == "/c" {
                println!("Clearing current chat history");
                conversation_history.clear();
                current_file = None;
                continue;
            // SAVING CHAT HISTORY
            // TODO: Implement a feature that will distinguish between user input and LLM output
            // this will likely need to be handled in the way I handle conversation_history as I
            // just store everything as raw there.
            } else if question == "/s" {
                // if there is a current_file set we keep writing to that file
                let filename = if let Some(ref file) = current_file {
                    save_chat_history(
                        Some(file),
                        &bedrock_runtime_client,
                        &mut conversation_history,
                    )
                    .await?
                } else {
                    match save_chat_history(
                        None,
                        &bedrock_runtime_client,
                        &mut conversation_history,
                    )
                    .await
                    {
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
                println!("Chat history saved to: {}", filename.cyan());
                continue;
            } else if question == "/r" {
                match list_chat_histories() {
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
                        match load_chat_history(selected_history) {
                            // we load the filename and the content from the history so we can keep
                            // sasving to it
                            // TODO: Make this work with SerializableMessage
                            Ok((content, filename, existing_title, summary)) => {
                                conversation_history.messages = Some(content);
                                conversation_history.title = Some(existing_title.clone());
                                conversation_history.summary = Some(summary.clone());
                                current_file = Some(filename);
                                utils::print_warning("----------------------------------------");
                                println!("Loaded chat history from: {}", selected_history.yellow());
                                println!();
                                println!("Loaded chat summary: ");
                                println!("{}", summary);
                                print_conversation_history(&conversation_history);
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
                utils::print_warning("/s\t \t - (BETA) Save chat history");
                utils::print_warning("/r\t \t - (BETA) Recall and load a chat history");
                utils::print_warning(
                    "/h\t \t - (BETA) Export history as HTML(saves in current dir)",
                );
                utils::print_warning("/q\t \t - Quit");
                continue;
            }
            // If we are looking at code - I need to include the user question in the first
            // message. Otherwise Bedrock keeps complaining about alternate messages between user
            // and assistant
            let message = if arguments.source.is_some() && message_count == 1 {
                let question_with_code = code
                    .as_ref()
                    .map(|src_code| format!("{}\n<question>{}</question>", src_code, question))
                    .unwrap_or_else(|| question.to_string());
                Message::builder()
                    .set_role(Some(ConversationRole::User))
                    .set_content(Some(vec![ContentBlock::Text(question_with_code)]))
                    .build()?
            } else {
                Message::builder()
                    .set_role(Some(ConversationRole::User))
                    .set_content(Some(vec![ContentBlock::Text(question.to_string())]))
                    .build()?
            };
            let mut messages = conversation_history.messages.unwrap_or_default().clone();
            messages.push(message.into());
            conversation_history.messages = Some(messages);

            println!("----------------------------------------");
            println!("☎️  | Calling Model: {}", &model_id);
            println!("----------------------------------------");

            let streamresp = call_converse_stream(
                &bedrock_runtime_client,
                model_id.to_string(),
                &conversation_history,
                inference_parameters.clone(),
                system_prompt,
            )
            .await?;

            // TODO: This can be a function
            let message = Message::builder()
                .set_role(Some(ConversationRole::Assistant))
                .set_content(Some(vec![ContentBlock::Text(streamresp.to_string())]))
                .build()?;
            let mut messages = conversation_history.messages.unwrap();
            messages.push(message.into());
            conversation_history.messages = Some(messages);
        }
    }

    Ok(())
}
