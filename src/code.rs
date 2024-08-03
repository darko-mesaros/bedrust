use std::{collections::HashMap, path::PathBuf};
use std::fs;
use anyhow::anyhow;
use aws_sdk_bedrockruntime::types::{ContentBlock, InferenceConfiguration};
use walkdir::{WalkDir, DirEntry};
use crate::constants;
use crate::models::converse::call_converse;

// NOTE:
// A few things to note here:
// - I am hardcoding the model that guesses the project type, meaning the customer will need to
// have that model enabled for this to work.
// - For larger projects we may reach the context size limit quite fast. So it is rather limited.
// - We need to provide to bits of information before the run commences:
//   - Size of the files that will be sent over
//   - Project type we assumed / file extensions being sent over
//

pub async fn code_chat(p: PathBuf, client: &aws_sdk_bedrockruntime::Client ) -> Result<String, anyhow::Error> {
    // === DEFAULT INFERENCE PARAMETERS ===
    // NOTE: Not sure if this is the best way to store this. Maybe also as part of a configuraiton
    // run
    // Maybe even have specific parameters for just guessing the code with a low temp value
    let inference_parameters: InferenceConfiguration = InferenceConfiguration::builder()
        .max_tokens(2048)
        .top_p(0.8)
        .temperature(0.5)
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
        let string: String = format!("\n<filename>{}</filename>\n<file_contents>{}\n</file_contents>", 
                filename.to_string_lossy(), 
                content);
        formatted_contents.push_str(string.as_str())
    }

    Ok(formatted_contents)
}

// a simple function to check if a file name is hidden (has a . in front)
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with('.'))
         .unwrap_or(false)
}

// Function to check if the file has one of the desired extensions
fn has_desired_extension(entry: &DirEntry, extensions: &[String]) -> bool {
    entry.file_type().is_file() && entry.path().extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.into()))
        .unwrap_or(false)
}

// Function to filter out specific paths
fn is_not_ignored(entry: &DirEntry) -> bool {
    let path = entry.path();
    let ignored_paths = constants::CODE_IGNORE_DIRS;
    !ignored_paths.iter().any(|ignored| path.to_string_lossy().contains(ignored))
}

// gets all files of a give filename in a given dir up to a certain depth
fn get_all_files(p: &PathBuf, ext: Option<Vec<String>>, l: u8) -> Result<Vec<PathBuf>, anyhow::Error> {
    if !p.exists() {
        return Err(anyhow!("🔴 | The specified path does not exist. Sorry!"));
    }

    let files: Vec<_> = WalkDir::new(p)
        .max_depth(l.into())
        .into_iter()
        .filter_entry(is_not_ignored)
        .filter_map(Result::ok)
        .filter(|entry| 
            {
                if ext.is_some() { 
                    let extensions = ext.clone().unwrap();
                    has_desired_extension(entry, &extensions) && !is_hidden(entry) 
                } else {
                    // FIX: Seems to return hidden files as well
                    !is_hidden(entry)
                }
            }
        )
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

async fn guess_code_type(files: Vec<PathBuf>, client: &aws_sdk_bedrockruntime::Client, inf_param: InferenceConfiguration) -> Result<Vec<String>, anyhow::Error> {
    // question
    let mut query = String::new();
    query.push_str(constants::PROJECT_GUESS_PROMPT);
    for file in files {
        query.push_str(file.into_os_string().to_str().unwrap());
    }

    let model_id = constants::PROJECT_GUESS_MODEL_ID;
    // let bcall = mk_bedrock_call(&query, None, model_id)?;
    // FIX: This just prints out the files - as this is how the call_bedrock function works
    // This println! is here to just make it look nice
    println!("Including the following file extensions in this run: ");
    let content = ContentBlock::Text(query);
    let response = call_converse(
        client,
        model_id.to_string(),
        inf_param,
        content,
        None,
    ).await?;
    let extensions: Vec<String> = serde_json::from_str(&response)?;
    // TODO: Have the ability to parse the response if its not an array - give it a chance to
    // "THINK"

    Ok(extensions)
}

fn get_file_contents(files: Vec<PathBuf>) -> Result<HashMap<PathBuf, String>, anyhow::Error> {

    let mut code = HashMap::new();
    for file in files { 
        let contents = fs::read_to_string(&file).unwrap();
        code.insert(
            file,
            contents,
        );
    }

    Ok(code)
}
