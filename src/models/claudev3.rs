use serde::{Serialize, Deserialize};
use aws_sdk_bedrockruntime::primitives::Blob;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeV3Config {
    pub anthropic_version: String,
    pub max_tokens: i32,
    pub role: String,
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3Body {
    pub anthropic_version: String,
    pub max_tokens: i32,
    pub messages: Vec<ClaudeV3Message>
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3Message {
    pub role: String,
    pub content: Vec<ClaudeV3Content>
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3Content {
    // renaming content_type to type due to type being a keyword
    #[serde(rename="type")]
    pub content_type: String,
    pub text: String,
}

impl ClaudeV3Body {
    pub fn new(anthropic_version: String, max_tokens: i32, role: String, content_type: String, text: String) -> ClaudeV3Body {
        let content = ClaudeV3Content {
            content_type,
            text
        };
        let message = ClaudeV3Message {
            role,
            content: vec!(content)
        };
        ClaudeV3Body {
            anthropic_version,
            max_tokens,
            messages: vec!(message)
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

// NOTE: This is just dead code now, as we are not using structs to deserialize the data
// rather I am using serde_json::Value to return specific elements due to the complexity of 
// the response coming from Claude V3
#[derive(serde::Deserialize, Debug)]
pub struct ClaudeV3Response {
    pub delta: ClaudeV3ResponseContent,
}

#[derive(serde::Deserialize, Debug)]
pub struct ClaudeV3ResponseContent {
    // renaming content_type to type due to type being a keyword
    //#[serde(rename="type")]
    //pub completion_type: String,
    pub text: String,
}
