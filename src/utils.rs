use clap::Parser;
use figlet_rs::FIGfont;

use std::{fs, io, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use serde::{Deserialize, Serialize};

// ######################################## ARGUMENT PARSING
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(value_enum)]
    #[arg(short, long)]
    pub model_id: ArgModels,
    #[arg(short, long)]
    pub caption: Option<PathBuf>,
    #[arg(short)]
    pub xml: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BedrustConfig {
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

pub fn hello_header(s: &str) -> io::Result<()> {
    let ansi_font = FIGfont::from_file("resources/ansishadow.flf").unwrap();
    let hello = ansi_font.convert(s);

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(255, 153, 0))))?;
    println!("{}", hello.unwrap());
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;

    Ok(())
}

pub fn load_bedrust_config(f: String) -> Result<BedrustConfig, anyhow::Error> {
    let file = fs::File::open(f)?;
    let config: BedrustConfig = ron::de::from_reader(file)?;
    Ok(config)
}
