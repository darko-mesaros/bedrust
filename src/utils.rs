use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use figlet_rs::FIGfont;

use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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
    #[arg(short, long)]
    pub model_id: Option<ArgModels>,

    #[arg(short, long)]
    pub caption: Option<PathBuf>,

    #[arg(short, long)]
    pub source: Option<PathBuf>,

    #[arg(short)]
    pub xml: bool,
}

#[derive(clap::ValueEnum, Clone, Serialize, Deserialize, Debug, Copy)]
pub enum ArgModels {
    Llama270b,
    Llama31405bInstruct,
    Llama3170bInstruct,
    Llama318bInstruct,
    CohereCommand,
    ClaudeV2,
    ClaudeV21,
    ClaudeV3Opus,
    ClaudeV3Sonnet,
    ClaudeV3Haiku,
    ClaudeV35Sonnet,
    ClaudeV352Sonnet,
    ClaudeV37Sonnet,
    ClaudeV35Haiku,
    Jurrasic2Ultra,
    DeepSeekR1,
    TitanTextExpressV1,
    Mixtral8x7bInstruct,
    Mistral7bInstruct,
    MistralLarge,
    MistralLarge2,
    NovaMicro,
    NovaLite,
    NovaPro,
}

impl Display for ArgModels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

// TODO: Implement FromStr for ArgModels to make it even more robust

impl ArgModels {
    pub fn to_str(&self) -> &'static str {
        match self {
            ArgModels::ClaudeV2 => "anthropic.claude-v2",
            ArgModels::ClaudeV21 => "anthropic.claude-v2:1",
            ArgModels::ClaudeV3Haiku => "anthropic.claude-3-haiku-20240307-v1:0",
            ArgModels::ClaudeV35Haiku => "anthropic.claude-3-5-haiku-20241022-v1:0",
            ArgModels::ClaudeV3Sonnet => "anthropic.claude-3-sonnet-20240229-v1:0",
            ArgModels::ClaudeV3Opus => "anthropic.claude-3-opus-20240229-v1:0",
            ArgModels::ClaudeV35Sonnet => "anthropic.claude-3-5-sonnet-20240620-v1:0",
            ArgModels::ClaudeV352Sonnet => "anthropic.claude-3-5-sonnet-20241022-v2:0",
            ArgModels::ClaudeV37Sonnet => "us.anthropic.claude-3-7-sonnet-20250219-v1:0",
            ArgModels::DeepSeekR1 => "us.deepseek.r1-v1:0",
            ArgModels::Llama270b => "meta.llama2-70b-chat-v1",
            ArgModels::Llama31405bInstruct => "meta.llama3-1-405b-instruct-v1:0",
            ArgModels::Llama3170bInstruct => "meta.llama3-1-70b-instruct-v1:0",
            ArgModels::Llama318bInstruct => "meta.llama3-1-8b-instruct-v1:0",
            ArgModels::CohereCommand => "cohere.command-text-v14",
            ArgModels::Jurrasic2Ultra => "ai21.j2-ultra-v1",
            ArgModels::TitanTextExpressV1 => "amazon.titan-text-express-v1",
            ArgModels::Mixtral8x7bInstruct => "mistral.mixtral-8x7b-instruct-v0:1",
            ArgModels::Mistral7bInstruct => "mistral.mistral-7b-instruct-v0:2",
            ArgModels::MistralLarge => "mistral.mistral-large-2402-v1:0",
            ArgModels::MistralLarge2 => "mistral.mistral-large-2407-v1:0",
            ArgModels::NovaMicro => "us.amazon.nova-micro-v1:0",
            ArgModels::NovaLite => "us.amazon.nova-lite-v1:0",
            ArgModels::NovaPro => "us.amazon.nova-pro-v1:0",
        }
    }

    pub fn from_config_str(s: &str) -> Option<Self> {
        match s {
            "anthropic.claude-v2" => Some(ArgModels::ClaudeV2),
            "anthropic.claude-v2:1" => Some(ArgModels::ClaudeV21),
            "anthropic.claude-3-haiku-20240307-v1:0" => Some(ArgModels::ClaudeV3Haiku),
            "anthropic.claude-3-5-haiku-20241022-v1:0" => Some(ArgModels::ClaudeV35Haiku),
            "anthropic.claude-3-sonnet-20240229-v1:0" => Some(ArgModels::ClaudeV3Sonnet),
            "anthropic.claude-3-opus-20240229-v1:0" => Some(ArgModels::ClaudeV3Opus),
            "anthropic.claude-3-5-sonnet-20240620-v1:0" => Some(ArgModels::ClaudeV35Sonnet),
            "anthropic.claude-3-5-sonnet-20241022-v2:0" => Some(ArgModels::ClaudeV352Sonnet),
            "us.anthropic.claude-3-7-sonnet-20250219-v1:0" => Some(ArgModels::ClaudeV37Sonnet),
            "us.deepseek.r1-v1:0" => Some(ArgModels::DeepSeekR1),
            "meta.llama2-70b-chat-v1" => Some(ArgModels::Llama270b),
            "meta.llama3-1-405b-instruct-v1:0" => Some(ArgModels::Llama31405bInstruct),
            "meta.llama3-1-70b-instruct-v1:0" => Some(ArgModels::Llama3170bInstruct),
            "meta.llama3-1-8b-instruct-v1:0" => Some(ArgModels::Llama318bInstruct),
            "cohere.command-text-v14" => Some(ArgModels::CohereCommand),
            "ai21.j2-ultra-v1" => Some(ArgModels::Jurrasic2Ultra),
            "amazon.titan-text-express-v1" => Some(ArgModels::TitanTextExpressV1),
            "mistral.mixtral-8x7b-instruct-v0:1" => Some(ArgModels::Mixtral8x7bInstruct),
            "mistral.mistral-7b-instruct-v0:2" => Some(ArgModels::Mistral7bInstruct),
            "mistral.mistral-large-2402-v1:0" => Some(ArgModels::MistralLarge),
            "mistral.mistral-large-2407-v1:0" => Some(ArgModels::MistralLarge2),
            "us.amazon.nova-micro-v1:0" => Some(ArgModels::NovaMicro),
            "us.amazon.nova-lite-v1:0" => Some(ArgModels::NovaLite),
            "us.amazon.nova-pro-v1:0" => Some(ArgModels::NovaPro),
            _ => None,
        }
    }
}
// ######################################## END ARGUMENT PARSING

pub fn hello_header(s: &str) -> Result<(), anyhow::Error> {
    let home_dir = home_dir().expect("Failed to get HOME directory");
    let config_dir = home_dir.join(format!(".config/{}", constants::CONFIG_DIR_NAME));
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    // test if show_banner is true
    if crate::config::load_bedrust_config()?.show_banner {
        let figlet_font_file_path = config_dir.join(constants::FIGLET_FONT_FILENAME);
        let figlet_path_str = figlet_font_file_path
            .as_path()
            .to_str()
            .ok_or_else(|| anyhow!("Was unable to parse Figlet font path to string"))?;
        let ansi_font = FIGfont::from_file(figlet_path_str)
            .map_err(|e| anyhow!("Failed to load the Figlet font: {}",e))?;
        let hello = ansi_font.convert(s)
            .ok_or_else(||anyhow!("Was unable to convert the hello message to ANSI fonts"))?;

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(255, 153, 0))))?;
        println!("{}", hello);
    } // if its false - just continue
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    println!("{}", "----------------------------------------".cyan());
    println!(
        "{}",
        "Currently supported chat commands: ".truecolor(83, 82, 82)
    );
    println!(
        "{}",
        "/c\t - Clear current chat history".truecolor(255, 229, 153)
    );
    println!(
        "{}",
        "/s\t - (BETA) Save chat history".truecolor(255, 229, 153)
    );
    println!(
        "{}",
        "/r\t - (BETA) Recall and load a chat history".truecolor(255, 229, 153)
    );
    println!(
        "{}",
        "/h\t - (BETA) Export history as HTML(saves in current dir)".truecolor(255, 229, 153)
    );
    println!("{}", "/q\t - Quit".truecolor(255, 229, 153));
    println!("{}", "----------------------------------------".cyan());
    println!();
    // NOTE: This println! statement is used to advertise new features
    // (This could probably be a nicer function)
    // Removed the CHAT enabled notification
    // println!(
    //     "{}{}{} 💬",
    //     "Now with ".italic(),
    //     "CHAT".red().on_yellow().blink(),
    //     " enabled!".italic()
    // );

    Ok(())
}

pub fn print_warning(s: &str) {
    println!("{}", s.yellow());
}
// TODO: Implement checking for AWS credentials

pub fn prompt_for_model_selection() -> Result<ArgModels, anyhow::Error> {
    let model_list = ArgModels::value_variants();
    let idx = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a model to use:")
        .items(model_list)
        .interact()?;
    Ok(model_list[idx])
}

pub fn prompt_for_model_selection_opt() -> Result<Option<ArgModels>, anyhow::Error> {
    let model_list = ArgModels::value_variants();
    let idx = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a default model to use press <enter> to skip")
        .items(model_list)
        .interact_opt()?;
    Ok(idx.map(|idx| model_list[idx]))
}
