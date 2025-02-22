use crate::constants;
use crate::models::converse::call_converse;
use crate::utils::print_warning;
use anyhow::anyhow;
use aws_sdk_bedrockruntime::types::{ContentBlock, InferenceConfiguration};
use ignore::DirEntry;
use std::fs;
use std::{collections::HashMap, path::PathBuf};

// NOTE:
// A few things to note here:
// - I am hardcoding the model that guesses the project type, meaning the customer will need to
// have that model enabled for this to work.
// - For larger projects we may reach the context size limit quite fast. So it is rather limited.
// - We need to provide to bits of information before the run commences:
//   - Size of the files that will be sent over
//   - Project type we assumed / file extensions being sent over

// This starts a process of the code chat. Moved here instead of being in the main.rs file
pub async fn code_chat_process(
    code_path: PathBuf,
    bedrock_runtime_client: &aws_sdk_bedrockruntime::Client,
) -> Result<String, anyhow::Error> {
    println!("----------------------------------------");
    print_warning("⚠ THIS IS A BETA FEATURE ⚠");
    println!("----------------------------------------");
    println!("💾 | Ooh, it Seems we are talking about code today!");
    println!(
        "💾 | I was given this dir to review: {:?}",
        &code_path // NOTE: How to print it here without a clone?
            .clone()
            .into_os_string()
    );
    println!("----------------------------------------");
    let mut convo = String::new();
    let code = code_chat(code_path.clone().to_path_buf(), bedrock_runtime_client).await?;

    // NOTE: Here is something stupid for my edge case
    let (p1, p2) = ("<bedrust_be", "gin_source>");
    let (p3, p4) = ("</bedrust_en", "d_source>");
    let wrapped_code = format!("{}{}{}{}{}", p1, p2, code, p3, p4);

    // Check if `.bedrustrules` exists
    let br_path = code_path.join(constants::INSTRUCTION_FILE);
    let query = match br_path.exists() && br_path.is_file() {
        true => {
            // It's here load it instead of the default system prompt
            let mut br_rules = fs::read_to_string(br_path)?;
            br_rules.push_str(
                r#"
Here are the files:
<SOURCE_CODE_BEDRUST>{SOURCE_CODE}</SOURCE_CODE_BEDRUST>
"#,
            );
            br_rules.replace("{SOURCE_CODE}", wrapped_code.as_str())
        }
        false => {
            // Nope, just use the default system prompt
            constants::CODE_CHAT_PROMPT.replace("{SOURCE_CODE}", wrapped_code.as_str())
        }
    };

    println!("----------------------------------------");
    print_warning("⚠ THIS IS A BETA FEATURE ⚠");

    // Return this conversation
    convo.push_str(query.as_str());

    Ok(convo)
}

pub async fn code_chat(
    p: PathBuf,
    client: &aws_sdk_bedrockruntime::Client,
) -> Result<String, anyhow::Error> {
    // === DEFAULT INFERENCE PARAMETERS ===
    // NOTE: Not sure if this is the best way to store this. Maybe also as part of a configuraiton
    // run
    // Maybe even have specific parameters for just guessing the code with a low temp value
    let inference_parameters: InferenceConfiguration = InferenceConfiguration::builder()
        .max_tokens(2048)
        .top_p(0.8)
        .temperature(0.2)
        .build();

    // FIGURE OUT PROJECT
    // FIX: Seems to return hidden files too
    let all_files = get_all_files(&p, None, 3)?;
    let extn = guess_code_type(all_files, client, inference_parameters).await?;

    // get all files with the extensions from above, and go 2 levels deep
    let files = get_all_files(&p, Some(extn), 3)?;
    let contents = get_file_contents(files)?;

    let mut formatted_contents = String::new();
    for (filename, content) in &contents {
        let string: String = format!(
            "\n<filename>{}</filename>\n<file_contents>{}\n</file_contents>",
            filename.to_string_lossy(),
            content
        );
        formatted_contents.push_str(string.as_str())
    }

    Ok(formatted_contents)
}

// a simple function to check if a file name is hidden (has a . in front)
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

// gets all files of a give filename in a given dir up to a certain depth
fn get_all_files(
    p: &PathBuf,
    ext: Option<Vec<String>>,
    l: u8,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    if !p.exists() {
        return Err(anyhow!("🔴 | The specified path does not exist. Sorry!"));
    }

    let mut builder = ignore::WalkBuilder::new(p);
    builder.max_depth(Some(l as usize));
    builder.hidden(false);

    let walker = builder.build();

    let files: Vec<_> = walker
        .filter_map(Result::ok)
        .filter(|entry| {
            let is_file = entry.file_type().is_some_and(|ft| ft.is_file());
            let matches_extension = ext.as_ref().is_none_or(|extensions| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| extensions.contains(&ext.to_string()))
            });
            let is_not_ignored = !is_hidden(entry);

            is_file && matches_extension && is_not_ignored
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

async fn guess_code_type(
    files: Vec<PathBuf>,
    client: &aws_sdk_bedrockruntime::Client,
    inf_param: InferenceConfiguration,
) -> Result<Vec<String>, anyhow::Error> {
    // question
    let mut query = String::new();
    query.push_str(constants::PROJECT_GUESS_PROMPT);
    for file in files {
        query.push_str(file.into_os_string().to_str().unwrap());
    }

    let model_id = constants::PROJECT_GUESS_MODEL_ID;
    // FIX: This just prints out the files - as this is how the call_bedrock function works
    // This println! is here to just make it look nice
    println!("Including the following file extensions in this run: ");
    let content = ContentBlock::Text(query);
    // === RETRY MECHANISM ===
    let max_retries = 3;
    let mut retry_count = 0;
    while retry_count < max_retries {
        match call_converse(
            client,
            model_id.to_string(),
            inf_param.clone(),
            content.clone(),
            None,
            true,
        )
        .await
        {
            Ok(response) => {
                // check if the response is a valid array
                match serde_json::from_str::<Vec<String>>(&response) {
                    Ok(extensions) => return Ok(extensions),
                    Err(_) => {
                        println!("🔴 | Response from `guess_code_type` is not a valid array. Retrying ...");
                        retry_count += 1;
                    }
                }
            }
            Err(e) => {
                // if an error occurs, print it and retry
                println!("🔴 | Error: {}", e);
                retry_count += 1;
            }
        }
        // if we have retried max_retries times, return an error
        if retry_count >= max_retries {
            return Err(anyhow!(
                "Failed to get a response after {} retries",
                max_retries
            ));
        }
        // sleep for 2^retry_count seconds - exponential backoff
        tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(retry_count))).await;
        // === END RETRY MECHANISM ===
    }
    Err(anyhow!("Unexpected error in guess_code_type"))
}

fn get_file_contents(files: Vec<PathBuf>) -> Result<HashMap<PathBuf, String>, anyhow::Error> {
    let mut code = HashMap::new();
    for file in files {
        let contents = fs::read_to_string(&file).unwrap();
        code.insert(file, contents);
    }

    Ok(code)
}
