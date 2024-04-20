// This file contains constants (duh)

// CONFIGURATION FILES
pub static CONFIG_DIR_NAME: &str = "bedrust";
pub static MODEL_CONFIG_FILE_NAME: &str = "model_config.ron";
pub static BEDRUST_CONFIG_FILE_NAME: &str = "bedrust_config.ron";

// UPDATED: 2024-04-20
pub static MODEL_CONFIG_FILE: &str = r#"ModelConfigs(
  llama270b: (
    temperature: 1, 
    p: 0.1,
    max_gen_len: 1024,
  ),
  cohere_command:(
      max_tokens: 500,
      temperature: 1.0,
      p: 0.1,
      k: 1,
      stop_sequences: [],
      stream: true,
  ),
  claude_v2:(
      temperature: 1.0,
      p: 1.0,
      k: 250,
      max_tokens_to_sample: 500,
      stop_sequences: [],
  ),
  claude_v21:(
      temperature: 1.0,
      p: 1.0,
      k: 250,
      max_tokens_to_sample: 500,
      stop_sequences: [],
  ),
 claude_v3:(
      anthropic_version: "bedrock-2023-05-31",
      max_tokens: 1000,
      role: "user",
      type: "text",
  ),
  jurrasic_2_ultra:(
      temperature: 0.7,
      top_p: 1,
      max_tokens: 200,
      stop_sequences: [],
  ),
  titan_text_express_v1:(
      temperature: 0,
      top_p: 1,
      max_token_count: 8192,
      stop_sequences: [],
  ),
  mixtral_8x7b_instruct:(
      temperature: 0.5,
      top_p: 0.9,
      top_k: 200,
      max_tokens: 1024,
      stop: [],
  ),
  mistral_7b_instruct:(
      temperature: 0.5,
      top_p: 0.9,
      top_k: 200,
      max_tokens: 1024,
      stop: [],
  ),
  mistral_large:(
      temperature: 0.5,
      top_p: 0.9,
      top_k: 200,
      max_tokens: 1024,
      stop: [],
  )
)
"#;

// UPDATED: 2024-03-30
pub static BEDRUST_CONFIG_FILE: &str = r#"BedrustConfig(
  aws_profile: "default",
  supported_images: ["jpg", "jpeg", "png", "bmp",],
  caption_prompt: "Please caption the following image for the sake of accessibility. Return just the caption, and nothing else. Keep it clean, and under 100 words."
)
"#;
// FIGLET FONT
pub static FIGLET_FONT_FILENAME: &str = "ansishadow.flf";
pub const FIGLET_FONT: &str = include_str!("../resources/ansishadow.flf");
