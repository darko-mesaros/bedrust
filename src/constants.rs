// This file contains constants (duh)

// PROMPTS
pub static CODE_CHAT_PROMPT: &str = r#"
You are my coding assistant and an expert in all things coding.
I have some code files that I'd like to discuss with you. Each file is provided in the following format:
\n<filename>filename</filename>\n<file_contents>filecontents</file_contents>

Please prepare to analyze the provided code, keeping in mind the following objectives for future questions:
1. **Code Review**: Identify any potential issues, bugs, or inefficiencies in the code. Be ready to suggest improvements or optimizations where necessary.
2. **Functionality Explanation**: Be prepared to explain the functionality of the code. What does each file or significant section of the code do?
3. **Best Practices**: Evaluate whether the code follows best practices in terms of style, structure, and design patterns. Be ready to recommend any changes that could enhance the code quality.
4. **Specific Questions**: I will have specific questions related to certain parts of the code. Please be prepared to provide detailed answers and examples if needed. Those questions will come after you have been provided the files.

Here are the files:
"#;

// NOTE: When using Claude you can use the Agent prompt to just finalize the array - Thank you
// Thiago <3
// This means I can start an array and it should finish it for me
pub static PROJECT_GUESS_PROMPT: &str = r#"
You are helping me figure out what kind of software development projects people are working on. To figure this out, you will look at a file structure of a directory and return to me an array of important file names related to that project type. You will only respond with that array and nothing else. Only return file types that are text files (do not return file types that are images or binaries)

Here is the example of such an array:
["rs","toml","md","ron"]

Give me an array of important files for a project type that has the following directory items:
"#;

// HELPER CONSTANTS
// FIX: the model id is hardcoded, we need to make this configurable
pub static PROJECT_GUESS_MODEL_ID: &str = "anthropic.claude-3-sonnet-20240229-v1:0";
pub static CODE_IGNORE_DIRS: &[&str] = &[
    // Rust
    "target",
    // JavaScript/TypeScript
    "node_modules",
    "dist",
    "build",
    // Python
    "__pycache__",
    ".tox",
    "venv",
    ".pytest_cache",
    // Java
    "target",
    "bin",
    ".gradle",
    ".mvn",
    // C/C++
    "obj",
    "out",
    // Go
    "pkg",
    // Ruby
    ".bundle",
    "vendor/bundle",
    // Django
    "staticfiles",
    "media",
    // General
    ".git",
    ".svn",
    ".hg",
    ".idea",
    ".vscode",
    ".DS_Store",
    "logs",
    "tmp",
    "cache",
    ".terraform",
];

// CONFIGURATION FILES
pub static CONFIG_DIR_NAME: &str = "bedrust";
pub static MODEL_CONFIG_FILE_NAME: &str = "model_config.ron";
pub static BEDRUST_CONFIG_FILE_NAME: &str = "bedrust_config.ron";

// UPDATED: 2024-08-02
pub static BEDRUST_CONFIG_FILE: &str = r#"BedrustConfig(
  // define what AWS profile to use
  aws_profile: "default",
  // what image extensions do you wish to support when running captioning
  supported_images: ["jpg", "jpeg", "png", "bmp",],
  // the prompt being used for image captioning
  caption_prompt: "Please caption the following image for the sake of accessibility. Return just the caption, and nothing else. Keep it clean, and under 100 words.",
  // choose to show the big ASCII banner on startup or not
  show_banner: true,
  inference_params: (
    temperature: 0.5,
    max_tokens: 2048,
    top_p: 0.8, 
  ),
)
"#;
// FIGLET FONT
pub static FIGLET_FONT_FILENAME: &str = "ansishadow.flf";
pub const FIGLET_FONT: &str = include_str!("../resources/ansishadow.flf");
