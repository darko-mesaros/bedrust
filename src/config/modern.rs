use anyhow::{Ok, Result};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::utils::{self, ArgModels};
use crate::constants;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BedrustConfig {
    pub aws: AwsConfig,
    pub app: AppConfig,
    pub inference: InferenceConfig,
    pub captioning: CaptioningConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AwsConfig {
    pub profile: String,
    pub region: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub default_model: Option<ArgModels>,
    pub show_banner: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InferenceConfig {
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CaptioningConfig {
    pub supported_images: Vec<String>,
    pub caption_prompt: String,
}

impl BedrustConfig {

    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = dirs::config_dir()
            .map(|d|d.join("bedrust")) // TODO: Have this as a constant
            .unwrap_or_else(|| PathBuf::from("config"));

        let default_model = utils::prompt_for_model_selection_opt().unwrap(); // FIX: Handle unwrap
        // TODO: IMPLEMENT A WAY TO SETUP A DEFAULT MODEL FROM THE PROMPT SELECTION

        let config = Config::builder()
            // load defaults
            .set_default("aws.profile", "default")?
            .set_default("aws.show_banner", true)?
            .set_default("inference.temperature", 0.5)?
            .set_default("inference.max_tokens", 2048)?
            .set_default("inference.top_p", 0.8)?
            .add_source(File::with_name(
                    config_dir.join("config.toml").to_str().unwrap()
            ).required(false))
            .add_source(Environment::with_prefix("BEDRUST").separator("_"))
            .build()?;

        config.try_deserialize()
    }

    fn default() -> Self {
        Self {
            aws: AwsConfig {
                profile: String::from("default"),
                region: None,
            },
            app: AppConfig {
                default_model: None,
                show_banner: true,
            },
            inference: InferenceConfig {
                temperature: constants::DEFAULT_TEMPERATURE,
                max_tokens: constants::DEFAULT_MAX_TOKENS,
                top_p: constants::DEFAULT_TOP_P,
            },
            captioning: CaptioningConfig {
                // Converts from &[&str] to Vec<String>
                supported_images: constants::DEFAULT_SUPPORTED_IMAGES
                    .iter()
                    .map(|&s| s.to_string())
                    .collect(),
                caption_prompt: String::from(constants::DEFAULT_CAPTION_PROMPT),
            }
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml = toml::to_string_pretty(&self)?;
        println!("DEBUG: {:?}", &toml);
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn load_or_create() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .map(|d|d.join("bedrust")) // TODO: Have this as a constant
            .ok_or_else(|| anyhow::anyhow!("Could not determine the config directory"))?;

        let config_path = config_dir.join("config.toml"); // TODO: Have this as a constant
        if !config_path.exists() {
            std::fs::create_dir_all(&config_dir)?;
            let default_config = Self::default();
            default_config.save(&config_path)?;
        }

        Ok(Self::new()?)
    }

    pub fn initialize(config_dir: PathBuf) -> Result<()> {
        std::fs::create_dir_all(&config_dir)?;
        let default_config = Self::default();
        default_config.save(&config_dir.join("config.toml"))?;

        Ok(())
    }

    pub fn config_exists() -> Result<bool> {
        let config_path = dirs::config_dir()
            .map(|d|d.join("bedrust").join("config.toml")) // TODO: Have this as a constant
            .ok_or_else(|| anyhow::anyhow!("Could not determine the config directory"))?;

        Ok(config_path.exists())
    }

    pub fn prompt_init() -> Result<()> {
        use dialoguer::Confirm;
        use colored::*;
        if Self::config_exists()? {
            println!("{}","****************************************".yellow());
            println!("{}","WARNING".yellow());
            println!("You are about to overwrite the existing Bedrust configruation.");
            println!("{}","****************************************".yellow());
            if !Confirm::new()
                .with_prompt("Are you sure you want to continue?")
                .interact()?
            {
                return Ok(());
            }
        }
        let config_dir = dirs::config_dir()
            .map(|d| d.join("bedrust"))
            .ok_or_else(||anyhow::anyhow!("Could not determine config directory"))?;

        println!("📜 | Initializing Bedrust configuration...");
        Self::initialize(config_dir)?;
        println!("✅ | Configuration initialized successfully!");

        Ok(())
    }

}
