use anyhow::anyhow;
use clap::Parser;
use figlet_rs::FIGfont;

use std::{fs, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use serde::{Deserialize, Serialize};

use colored::*;

use dirs::home_dir;

use crate::constants;

// ######################################## ARGUMENT PARSING
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, conflicts_with("model_id"))]
    pub init: bool,

    #[clap(value_enum)]
    #[arg(short, long, required_unless_present("init"))]
    pub model_id: Option<ArgModels>,

    #[arg(short, long)]
    pub caption: Option<PathBuf>,

    #[arg(short)]
    pub xml: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BedrustConfig {
    pub aws_profile: String,
    pub supported_images: Vec<String>,
    pub caption_prompt: String,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ArgModels {
    Llama270b,
    CohereCommand,
    ClaudeV2,
    ClaudeV21,
    ClaudeV3Sonnet,
    ClaudeV3Haiku,
    Jurrasic2Ultra,
    TitanTextExpressV1,
    Mixtral8x7bInstruct,
    Mistral7bInstruct,
}

impl ArgModels {
    pub fn to_str(&self) -> &'static str {
        match self {
            ArgModels::ClaudeV2 => "anthropic.claude-v2",
            ArgModels::ClaudeV21 => "anthropic.claude-v2:1",
            ArgModels::ClaudeV3Haiku => "anthropic.claude-3-haiku-20240307-v1:0",
            ArgModels::ClaudeV3Sonnet => "anthropic.claude-3-sonnet-20240229-v1:0",
            ArgModels::Llama270b => "meta.llama2-70b-chat-v1",
            ArgModels::CohereCommand => "cohere.command-text-v14",
            ArgModels::Jurrasic2Ultra => "ai21.j2-ultra-v1",
            ArgModels::TitanTextExpressV1 => "amazon.titan-text-express-v1",
            ArgModels::Mixtral8x7bInstruct => "mistral.mixtral-8x7b-instruct-v0:1",
            ArgModels::Mistral7bInstruct => "mistral.mistral-7b-instruct-v0:2",
        }
    }
}
// ######################################## END ARGUMENT PARSING

pub fn hello_header(s: &str) -> Result<(), anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    let figlet_font_file_path = config_dir.join(constants::FIGLET_FONT_FILENAME);
    let figlet_path_str = figlet_font_file_path
        .as_path()
        .to_str()
        .ok_or_else(||anyhow!("Was unable to parse Figlet font path to string"))?;
    let ansi_font = FIGfont::from_file(figlet_path_str).unwrap();
    let hello = ansi_font.convert(s);

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(255, 153, 0))))?;
    println!("{}", hello.unwrap());
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    println!("{}", "----------------------------------------".cyan());
    println!("{}", "Currently supported chat commands: ".truecolor(83,82,82));
    println!("{}", "/q\t \t - Quit".truecolor(255, 229, 153));
    println!("{}", "----------------------------------------".cyan());
    println!("{}{}{} 💬", "Now with ".italic(), "CHAT".red().on_yellow().blink(), " enabled!".italic());

    Ok(())
}

pub fn load_bedrust_config() -> Result<BedrustConfig, anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    let bedrust_config_file_path = config_dir.join(constants::BEDRUST_CONFIG_FILE_NAME);

    let file = fs::File::open(bedrust_config_file_path)?;
    let config: BedrustConfig = ron::de::from_reader(file)?;
    Ok(config)
}

pub fn print_warning(s: &str) {
    println!("{}", s.yellow());
}
// TODO: Implement checking for AWS credentials

// function that checks if there are any configuration files present
pub fn check_for_config() -> Result<bool, anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(".config/bedrust");
    let bedrust_config_file_path = config_dir.join("bedrust_config.ron");
    let model_config_file_path = config_dir.join("model_config.ron");

    if !bedrust_config_file_path.exists() || !model_config_file_path.exists() {
        Ok(false)
    } else { 
        Ok(true)
    }

}

// function that creates the configuration files during the `init` command
pub fn initialize_config() -> Result<(), anyhow::Error>{
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    fs::create_dir_all(&config_dir)?;

    let bedrust_config_file_path = config_dir.join(constants::BEDRUST_CONFIG_FILE_NAME);
    let bedrust_config_content = constants::BEDRUST_CONFIG_FILE.to_string();
    fs::write(&bedrust_config_file_path, bedrust_config_content)?;
    println!("⏳| Bedrust configuration file created at: {:?}", bedrust_config_file_path);
    println!("This file is used to store configuration items for the bedrust application.");

    let model_config_file_path = config_dir.join(constants::MODEL_CONFIG_FILE_NAME);
    let model_config_content = constants::MODEL_CONFIG_FILE.to_string();
    fs::write(&model_config_file_path, model_config_content)?;
    println!("⏳| Model configuration file created at: {:?}", model_config_file_path);
    println!("This file is used to store default model parameters such as max tokens, temperature, top_p, top_k, etc.");

    let figlet_font_file_path = config_dir.join(constants::FIGLET_FONT_FILENAME);
    let figlet_font_content = constants::FIGLET_FONT;
    fs::write(&figlet_font_file_path, figlet_font_content)?;
    println!("⏳| Figlet font created at: {:?}", figlet_font_file_path);
    println!("This file is used to as a font for `figlet` to create the nice big font during launch.");

    println!("✅ | Bedrust configuration has been initialized in ~/.config/bedrust. You may now use it as normal.");
    Ok(())
}

















