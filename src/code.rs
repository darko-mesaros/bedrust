use std::{collections::HashMap, path::PathBuf};
use std::fs;
use walkdir::{WalkDir, DirEntry};
use crate::constants;
use crate::RunType;
use crate::mk_bedrock_call;
use crate::call_bedrock;

pub async fn code_chat(p: PathBuf, client: &aws_sdk_bedrockruntime::Client ) -> Result<String, anyhow::Error> {
    // FIGURE OUT PROJECT
    // FIX: Seems to return hidden files too
    let all_files = get_all_files(&p, None, 2)?;
    println!("{:#?}", all_files);
    let extn = guess_code_type(all_files, client).await?;
    println!("EXTN: {:#?}", extn);
    
    //let ext = vec!["rs", "md", "toml", "ron"];
    // get all files with the extensions from above, and go 2 levels deep
    let files = get_all_files(&p, Some(extn), 2)?;
    let contents = get_file_contents(files)?;
    // println!("CONTENTS: {:#?}", contents);

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
fn has_desired_extension(entry: &DirEntry, extensions: &Vec<String>) -> bool {
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

async fn guess_code_type(files: Vec<PathBuf>, client: &aws_sdk_bedrockruntime::Client) -> Result<Vec<String>, anyhow::Error> {
    // question
    let mut query = String::new();
    query.push_str(constants::PROJECT_GUESS_PROMPT);
    for file in files {
        query.push_str(file.into_os_string().to_str().unwrap());
    }

    // model_id
    let model_id = constants::PROJECT_GUESS_MODEL_ID;
    // client
    // run_type?
    let bcall = mk_bedrock_call(&query, None, model_id)?;
    // FIX: This just prints out the files
    let response = call_bedrock(client, bcall, RunType::Standard).await?;
    let extensions: Vec<String> = serde_json::from_str(&response)?;

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
