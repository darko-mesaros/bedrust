use std::fs;
use std::path::{PathBuf, Path};
use walkdir::WalkDir;

// add:
// examine directory structure and determine the project type
fn get_directory_tree(dir_path: &PathBuf) -> Result<String, anyhow::Error>{
    // SKIP NODE_MODULES
    let skip_dirs = ["node_modules"];
    let walker = WalkDir::new(dir_path)
        .into_iter()
        .filter_entry(|entry| !skip_dirs.contains(&entry.file_name().to_string_lossy().as_ref()));

    let mut directory_structure = String::new();

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        let depth = path.components().count() - dir_path.components().count();
        let prefix = "  ".repeat(depth);

        if path.is_dir() {
            directory_structure.push_str(&format!("{}{}/", prefix, path.file_name().unwrap().to_string_lossy()));
        } else {
            directory_structure.push_str(&format!("{}{}", prefix, path.file_name().unwrap().to_string_lossy()));
        }
    }
    Ok(directory_structure)
}

pub fn examine_source_dir(dir_path: &PathBuf) -> Result<Vec<String>, anyhow::Error> {
    let examine_prompt = "Here is a directory structure of my source code. Take a look at it and determine what file types should I search for to extract all of the source code from the files programatically. Provide me just the extensions I need to search in a Rust Vector, and nothing else:";
    // get dir tree
    // call model
    // return result
    // verify Result
    // if cannot determine, prompt user for file extensions
    //
    // PROBLEM: The model does not always return the same value. Sometimes it's quite chatty. How
    // can I fix that? Is there a better prompt? 
    // Do I store standard projects (bash, rust, js, react, cdk, cloudformation, python, .NET)?

    Ok(Vec::new())
}

// used  for reading source code files




pub fn read_files_of_type(dir_path: &PathBuf, file_extension: &str) -> Result<String, std::io::Error> {
    let mut file_contents = String::new();

    if dir_path.is_dir() {
        traverse_dir(&dir_path, &mut file_contents, file_extension)?;
    }

    Ok(file_contents)
}

fn traverse_dir(
    dir_path: &PathBuf,
    file_contents: &mut String,
    file_extension: &str,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // FIX: Make this configurable
            if path.file_name().unwrap() == Path::new("node_modules") {
                // Skip the "node_modules" directory
                continue;
            }
            traverse_dir(&path, file_contents, file_extension)?;
        } else if path.is_file() && path.extension().unwrap_or_default() == file_extension {
            let content = fs::read_to_string(&path)?;
            file_contents.push_str(&format!(
                "\n\nFILENAME: {} \n\nCONTENT: \n\n{}\n\n-----",
                path.display(),
                content
            ));
        }
    }

    Ok(())
}

