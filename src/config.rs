use crate::utils::{print_warning, prompt_for_model_selection_opt, ArgModels};
use anyhow::anyhow;
use config::{Config, ConfigError, File, FileFormat};
use dirs::home_dir;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{io, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BedrustConfig {
    pub aws_profile: String,
    pub supported_images: Vec<String>,
    pub caption_prompt: String,
    pub default_model: Option<String>,
    #[serde(default = "_default_true")]
    pub show_banner: bool,
    pub inference_params: InferenceParams,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InferenceParams {
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
}

fn _default_true() -> bool {
    true
}

impl Default for BedrustConfig {
    fn default() -> Self {
        Self {
            aws_profile: "default".to_string(),
            supported_images: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string(), "bmp".to_string()],
            caption_prompt: "Please caption the following image for the sake of accessibility. Return just the caption, and nothing else. Keep it clean, and under 100 words.".to_string(),
            default_model: None,
            show_banner: true,
            inference_params: InferenceParams {
                temperature: 0.5,
                max_tokens: 2048,
                top_p: 0.8
            },
            system_prompt: Some("You are a helpful assistant.".to_string())
        }
    }
}

impl BedrustConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let home_dir = home_dir().expect("Failed to get HOME directory");
        let config_dir = home_dir.join(".config/bedrust");
        let config_file = config_dir.join("bedrust_config.ron");
        // ~/.config/bedrust/bedrust_config.ron

        // Create a new config instance
        let config = Config::builder()
            // Start with default values
            .add_source(config::File::from_str(
                crate::constants::BEDRUST_CONFIG_FILE,
                FileFormat::Ron,
            ))
            // Add the config file if it already exists
            .add_source(
                File::from(config_file)
                    .required(false)
                    .format(FileFormat::Ron),
            )
            // Add in settings from the environment
            .add_source(config::Environment::with_prefix("BEDRUST").separator("__"))
            .build()?;

        let bedrust_config: BedrustConfig = config.try_deserialize()?;

        Ok(bedrust_config)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let home_dir = home_dir().expect("Failed to get HOME directory");
        let config_dir = home_dir.join(".config/bedrust");
        std::fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("bedrust_config.ron");
        let config_str = ron::ser::to_string_pretty(self, PrettyConfig::new())?;

        std::fs::write(config_file, config_str)?;

        Ok(())
    }

    pub fn exists() -> bool {
        let home_dir = home_dir().expect("Failed to get HOME directory");
        let config_file = home_dir.join(".config/bedrust/bedrust_config.ron");
        config_file.exists()
    }

    pub fn get_config_path() -> PathBuf {
        let home_dir = home_dir().expect("Failed to get HOME directory");
        home_dir.join(".config/bedrust")
    }

    pub fn get_default_model(&self) -> Option<ArgModels> {
        self.default_model
            .as_ref()
            .and_then(|s| ArgModels::from_config_str(s))
    }
}

pub fn prompt_init_config() -> Result<(), anyhow::Error> {
    if BedrustConfig::exists() {
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

fn initialize_config() -> Result<(), anyhow::Error> {
    // Create default config
    let mut config = BedrustConfig::default();
    // Prompt for model selection
    let model_choice = prompt_for_model_selection_opt()?;
    config.default_model = match model_choice {
        Some(model) => Some(model.to_str().to_string()),
        None => std::process::exit(1), // fail if no choice - this will likely never happen
    };
    // Save the config
    config.save()?;

    // Create a figlet font file
    let config_dir = BedrustConfig::get_config_path();
    let figlet_font_file_path = config_dir.join(crate::constants::FIGLET_FONT_FILENAME);
    std::fs::write(&figlet_font_file_path, crate::constants::FIGLET_FONT)?;

    println!(
        "⏳| Bedrust configuration file created at: {:?}",
        config_dir.join("bedrust_config.ron")
    );
    println!("This file is used to store configuration items for the bedrust application.");
    println!("⏳| Figlet font created at: {:?}", figlet_font_file_path);
    println!(
        "This file is used to as a font for `figlet` to create the nice big font during launch."
    );
    println!("✅ | Bedrust configuration has been initialized in ~/.config/bedrust. You may now use it as normal.");

    Ok(())
}

pub fn load_bedrust_config() -> Result<BedrustConfig, anyhow::Error> {
    BedrustConfig::new().map_err(|e| anyhow!("Failed to load the configuration: {}", e))
}

pub fn check_for_config() -> Result<bool, anyhow::Error> {
    Ok(BedrustConfig::exists())
}
