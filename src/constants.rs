// This file contains constants (duh)
use aws_sdk_bedrockruntime::types::InferenceConfiguration;
use lazy_static::lazy_static;

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

Think about your answer, and ask questions for clarification if needed.

At the end there will an initial user question inside the <question></question> tags.

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

pub static CONVERSATION_TITLE_PROMPT: &str = r#"This is a conversation history between a human user and a large language model. Generate only a concise 4-6 word title for the following conversation history. The history is enclosed in the <CONVERSATON_HISTORY> tags. The title should use underscores instead of spaces, and be all in lowercase. Only characters allowed are text characters, numbers and underscore (_). Do not provide any additional text or explanation.

<CONVERSATON_HISTORY>
{}
</CONVERSATON_HISTORY>

Title:"#;

pub static CONVERSATION_SUMMARY_PROMPT: &str = r#"This is a conversation history from a human user and a large language model. Summarize the key points of the following conversation in a single, cohesive paragraph. The conversation is enclosed in the <CONVERSATON_HISTORY> tags. Do not use bullet points or numbered lists. Focus on the main topics discussed and any conclusions reached. Keep the summary concise, between 3-5 sentences. Provide only the summary paragraph, without any introductory phrases or explanations.

<CONVERSATON_HISTORY>
{}
</CONVERSATON_HISTORY>

Summary:"#;

// INFERENCE CONSTANTS
lazy_static! {
    pub static ref CONVERSATION_HISTORY_INF_PARAMS: InferenceConfiguration =
        InferenceConfiguration::builder()
            .max_tokens(256)
            .top_p(0.8)
            .temperature(0.2)
            .build();
}

lazy_static! {
    pub static ref CONVERSATION_HISTORY_TITLE_INF_PARAMS: InferenceConfiguration =
        InferenceConfiguration::builder()
            .max_tokens(32)
            .top_p(0.8)
            .temperature(0.2)
            .build();
}

// HELPER CONSTANTS
// FIX: the model id is hardcoded, we need to make this configurable
pub static PROJECT_GUESS_MODEL_ID: &str = "anthropic.claude-3-haiku-20240307-v1:0";
pub static CONVERSATION_HISTORY_MODEL_ID: &str = "anthropic.claude-3-haiku-20240307-v1:0";
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

// HTML TEMPLATE FOR EXPORT
pub static HTML_TEMPLATE: &str = r#"
<html>
    <head>
        <title>Bedrust chat export {{title}}</title>
        <style>
            .message {
                margin-bottom: 15px;
                border-left: 4px solid #ddd;
                padding-left: 10px;
            }
            .role {
                font-weight: bold;
                color: #333;
            }
            .content {
                color: #666;
                white-space: pre-wrap;
                word-wrap: break-word;
            }
        </style>
    </head>
    <body>
        <h2>{{title}}</h2>
        <p><b>Summary:</b></br>{{nl2br summary}}</p>
        <hr>
        <div>
        {{! Iterate over messages and display here }}
        {{#each messages}}
            <div class="role">{{this.role}}
                <div class="content"><b>Message:</b>{{{nl2br content}}}</div>
            </div>
            <hr>
        {{/each}}
        </div>
    </body>
</html>
"#;

pub static HTML_TW_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html class="bg-gray-50">
<head>
    <title>Bedrust chat export: {{format_title title}}</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.tailwindcss.com"></script>
    <!-- Add Prism.js for syntax highlighting -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism.min.css"/>
    <!-- <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.24.1/themes/prism-tomorrow.min.css"/> -->
    <style>
        /* Style for code blocks */
        .code-wrapper {
            position: relative;
        }
        
        .copy-button {
            position: absolute;
            right: 8px;
            top: 8px;
            display: none;
            align-items: center;
            padding: 4px 8px;
            font-size: 0.75rem;
            font-weight: 500;
            color: rgb(75 85 99);
            background-color: rgb(243 244 246);
            border-radius: 0.375rem;
            transition: all 200ms;
        }

        .code-wrapper:hover .copy-button {
            display: flex;
        }

        .copy-button:hover {
            background-color: rgb(229 231 235);
        }

        .copy-button svg {
            width: 1rem;
            height: 1rem;
            margin-right: 0.25rem;
        }

        /* Add padding to accommodate the copy button */
        pre[class*="language-"] {
            margin: 1em 0;
            border-radius: 0.375rem;
        }
        
        pre[class*="language-"] code {
            padding-top: 2.5rem !important;
        }
    </style>
</head>
<body class="min-h-screen p-4 md:p-8">
    <div class="max-w-4xl mx-auto bg-white rounded-lg shadow-lg overflow-hidden"> <!-- Added overflow-hidden -->
        <!-- New Gradient Title Section -->
        <div class="bg-gradient-to-r from-orange-400 via-pink-500 to-blue-500 p-8 relative">
            <div class="relative z-10">
                <h1 class="text-2xl md:text-3xl font-bold text-white mb-2 text-shadow">{{format_title title}}</h1>
                <div class="h-1 w-20 bg-white rounded-full opacity-75"></div>
            </div>
            <!-- Optional decorative elements -->
            <div class="absolute bottom-0 right-0 transform translate-y-1/2 translate-x-1/4 w-64 h-64 bg-white/10 rounded-full blur-xl"></div>
            <div class="absolute top-0 left-0 transform -translate-y-1/2 -translate-x-1/4 w-48 h-48 bg-white/10 rounded-full blur-lg"></div>
        </div>

        <!-- Content Container with new padding -->
        <div class="p-6">
            <!-- Summary Section -->
            <div class="bg-blue-50 rounded-lg p-6 mb-8">
                <h2 class="text-lg font-semibold text-blue-800 mb-2">Summary</h2>
                <p class="text-gray-700 leading-relaxed">
                    {{{nl2br_with_code summary}}}
                </p>
            </div>

            <!-- Messages Section -->
            <div class="space-y-6">
                {{#each messages}}
                <div class="rounded-lg {{#if (eq this.role 'user')}}bg-gray-50{{else}}bg-green-50{{/if}} p-4">
                    <!-- Role Badge -->
                    <div class="flex items-center mb-2">
                        <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium 
                            {{#if (eq this.role 'user')}}
                                bg-gray-200 text-gray-800
                            {{else}}
                                bg-green-200 text-green-800
                            {{/if}}">
                            {{this.role}}
                        </span>
                    </div>

                    <!-- Message Content -->
                    <div class="prose max-w-none">
                        {{{nl2br_with_code content}}}
                    </div>
                </div>
                {{/each}}
            </div>

            <!-- Timestamp -->
            <div class="mt-8 text-sm text-gray-500 text-right">
                Generated: {{timestamp}}
            </div>
        </div>
    </div>
    <script>
    // This script generates the Copy button for the code.
    document.addEventListener('DOMContentLoaded', () => {
        // Find all pre elements and wrap them
        document.querySelectorAll('pre').forEach(pre => {
            // Create wrapper
            const wrapper = document.createElement('div');
            wrapper.className = 'code-wrapper';
            
            // Create copy button
            const copyButton = document.createElement('button');
            copyButton.className = 'copy-button focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500';
            copyButton.innerHTML = `
                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                          d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3">
                    </path>
                </svg>
                Copy
            `;
            
            // Add click handler
            copyButton.addEventListener('click', async () => {
                const code = pre.querySelector('code');
                const originalText = copyButton.innerHTML;
                
                try {
                    await navigator.clipboard.writeText(code.innerText);
                    copyButton.innerHTML = `
                        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                                  d="M5 13l4 4L19 7">
                            </path>
                        </svg>
                        Copied!
                    `;
                    copyButton.classList.add('bg-green-100', 'text-green-800');
                    
                    setTimeout(() => {
                        copyButton.innerHTML = originalText;
                        copyButton.classList.remove('bg-green-100', 'text-green-800');
                    }, 2000);
                } catch (err) {
                    console.error('Failed to copy:', err);
                    copyButton.textContent = 'Error!';
                    setTimeout(() => {
                        copyButton.innerHTML = originalText;
                    }, 2000);
                }
            });
            
            // Insert the button and wrap the pre
            pre.parentNode.insertBefore(wrapper, pre);
            wrapper.appendChild(pre);
            wrapper.appendChild(copyButton);
        });

        // Initialize Prism
        Prism.highlightAll();
    });
    </script>

    <!-- Add Prism.js script -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js"></script>
</body>
</html>
"#;
