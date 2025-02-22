use crate::utils::{print_warning, prompt_for_model_selection_opt, ArgModels};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;

use crate::constants;
use ron::ser::PrettyConfig;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct BedrustConfig {
    pub aws_profile: String,
    pub supported_images: Vec<String>,
    pub caption_prompt: String,
    pub default_model: Option<ArgModels>,
    // NOTE: There must be a better way for configuration defaults
    // for now if there is no configuration line use true
    // This is because the Serde default is `false`
    #[serde(default = "_default_true")]
    pub show_banner: bool,
    pub inference_params: InferenceParams,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InferenceParams {
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
}

pub fn prompt_init_config() -> Result<(), anyhow::Error> {
    match check_for_config() {
        Ok(config) => match config {
            true => {
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
            }
            false => {
                println!("----------------------------------------");
                println!("📜 | Initializing Bedrust configuration.");
                initialize_config()?;
            }
        },
        Err(e) => eprintln!("There was an issue checking for errors: {}", e),
    }
    print_warning("Bedrust will now exit");
    std::process::exit(0);
}

// function that checks if there are any configuration files present
pub fn check_for_config() -> Result<bool, anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(".config/bedrust");
    let bedrust_config_file_path = config_dir.join("bedrust_config.ron");

    if !bedrust_config_file_path.exists() {
        Ok(false)
    } else {
        Ok(true)
    }
}

// function that creates the configuration files during the `init` command
pub fn initialize_config() -> Result<(), anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    fs::create_dir_all(&config_dir)?;

    let bedrust_config_file_path = config_dir.join(constants::BEDRUST_CONFIG_FILE_NAME);
    let bedrust_config_content = constants::BEDRUST_CONFIG_FILE.to_string();

    let mut default_config: BedrustConfig =
        ron::de::from_str(&bedrust_config_content).expect("default config must be valid");
    default_config.default_model = prompt_for_model_selection_opt()?;

    fs::write(
        &bedrust_config_file_path,
        ron::ser::to_string_pretty(&default_config, PrettyConfig::new())?,
    )?;
    println!(
        "⏳| Bedrust configuration file created at: {:?}",
        bedrust_config_file_path
    );
    println!("This file is used to store configuration items for the bedrust application.");

    let figlet_font_file_path = config_dir.join(constants::FIGLET_FONT_FILENAME);
    let figlet_font_content = constants::FIGLET_FONT;
    fs::write(&figlet_font_file_path, figlet_font_content)?;
    println!("⏳| Figlet font created at: {:?}", figlet_font_file_path);
    println!(
        "This file is used to as a font for `figlet` to create the nice big font during launch."
    );

    println!("✅ | Bedrust configuration has been initialized in ~/.config/bedrust. You may now use it as normal.");
    Ok(())
}

pub fn load_bedrust_config() -> Result<BedrustConfig, anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    let bedrust_config_file_path = config_dir.join(constants::BEDRUST_CONFIG_FILE_NAME);

    let file = fs::File::open(bedrust_config_file_path)?;
    //let config: BedrustConfig = ron::de::from_reader(file)?;
    let config: BedrustConfig = ron::de::from_reader(file)?;
    Ok(config)
}

// ######################################## CONST FUNCTIONS
// Used to set default values to struct fields during serialization
const fn _default_true() -> bool {
    true
}
// ######################################## END CONST FUNCTIONS
