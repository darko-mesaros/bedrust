use aws_sdk_bedrockruntime::{
    operation::converse::{ConverseError, ConverseOutput},
    types::{ContentBlock, ConversationRole, InferenceConfiguration, Message, SystemContentBlock},
};

// Converse Error type
//
#[derive(Debug)]
pub struct BedrockConverseError(String);
impl std::fmt::Display for BedrockConverseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIX:
        // Figure out how to have this
        // write!(f, "Can't invoke '{}'. Reason: {}", MODEL_ID, self.0)
        write!(f, "Can't invoke. Reason: {}", self.0)
    }
}
impl std::error::Error for BedrockConverseError {}
impl From<&str> for BedrockConverseError {
    fn from(value: &str) -> Self {
        BedrockConverseError(value.to_string())
    }
}
impl From<&ConverseError> for BedrockConverseError {
    fn from(value: &ConverseError) -> Self {
        BedrockConverseError::from(match value {
            ConverseError::ModelTimeoutException(_) => "Model took too long",
            ConverseError::ModelNotReadyException(_) => "Model is not ready",
            _ => "Unknown",
        })
    }
}
// === Main functions ===

// Function to get the output text
fn get_converse_output_text(output: ConverseOutput) -> Result<String, BedrockConverseError> {
    let text = output
        .output()
        .ok_or("no output")?
        .as_message()
        .map_err(|_| "output not a message")?
        .content()
        .first()
        .ok_or("no content in message")?
        .as_text()
        .map_err(|_| "content is not text")?
        .to_string();
    Ok(text)
}

pub async fn call_converse(
    bc: &aws_sdk_bedrockruntime::Client,
    model_id: String,
    inference_parameters: InferenceConfiguration,
    content: ContentBlock,
    system: Option<Vec<SystemContentBlock>>,
) -> Result<String, BedrockConverseError> {
    let response = bc
        .converse()
        .model_id(model_id)
        .set_system(system)
        .messages(
            Message::builder()
                .role(ConversationRole::User)
                // FIX: How to not clone this?
                .content(content.clone())
                .build()
                .map_err(|_| "Failed to build message")?,
        )
        .inference_config(inference_parameters)
        .send()
        .await;

    match response {
        Ok(output) => {
            let text = get_converse_output_text(output)?;
            if content.is_text() {
                println!("{}", text);
            }
            Ok(text)
        }
        Err(e) => Err(e
            .as_service_error()
            .map(BedrockConverseError::from)
            .unwrap_or_else(|| BedrockConverseError("Unknown service error".into()))),
    }
}
