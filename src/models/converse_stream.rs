use crate::chat::{Conversation, ConversationEntity, ConversationHistory};
use aws_sdk_bedrockruntime::{
    error::ProvideErrorMetadata,
    operation::converse_stream::ConverseStreamError,
    types::{
        error::ConverseStreamOutputError, ConverseStreamOutput as ConverseStreamOutputType,
        InferenceConfiguration, Message, SystemContentBlock,
    },
};

// Converse Error type
//
#[derive(Debug)]
pub struct BedrockConverseStreamError(String);
impl std::fmt::Display for BedrockConverseStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Can't invoke. Reason: {}", self.0)
    }
}
impl std::error::Error for BedrockConverseStreamError {}
impl From<&str> for BedrockConverseStreamError {
    fn from(value: &str) -> Self {
        BedrockConverseStreamError(value.into())
    }
}

impl From<&ConverseStreamError> for BedrockConverseStreamError {
    fn from(value: &ConverseStreamError) -> Self {
        BedrockConverseStreamError(
            match value {
                ConverseStreamError::ModelTimeoutException(_) => "Model took too long",
                ConverseStreamError::ModelNotReadyException(_) => "Model is not ready",
                ConverseStreamError::ThrottlingException(_) => {
                    "Your request was throttled, please check your service quotas"
                }
                _ => "Unknown",
            }
            .into(),
        )
    }
}

impl From<&ConverseStreamOutputError> for BedrockConverseStreamError {
    fn from(value: &ConverseStreamOutputError) -> Self {
        match value {
            ConverseStreamOutputError::ValidationException(ve) => BedrockConverseStreamError(
                ve.message().unwrap_or("Unknown ValidationException").into(),
            ),
            ConverseStreamOutputError::ThrottlingException(te) => BedrockConverseStreamError(
                te.message().unwrap_or("Unknown ThrottlingException").into(),
            ),
            value => BedrockConverseStreamError(
                value
                    .message()
                    .unwrap_or("Unknown StreamOutput exception")
                    .into(),
            ),
        }
    }
}

// === Main functions ===

// Function to get the output text
fn get_converse_output_text(
    output: ConverseStreamOutputType,
) -> Result<String, BedrockConverseStreamError> {
    Ok(match output {
        ConverseStreamOutputType::ContentBlockDelta(event) => match event.delta() {
            Some(delta) => delta.as_text().cloned().unwrap_or_else(|_| "".into()),
            None => "".into(),
        },
        _ => "".into(),
    })
}

pub async fn call_converse_stream(
    bc: &aws_sdk_bedrockruntime::Client,
    model_id: String,
    conversation_history: &ConversationHistory,
    inference_parameters: InferenceConfiguration,
    system_prompt: &str,
) -> Result<Conversation, BedrockConverseStreamError> {
    let msg: Vec<Message> = conversation_history
        .messages
        .clone()
        .unwrap()
        .into_iter()
        .map(Message::from)
        .collect();

    let response = bc
        .converse_stream()
        .model_id(model_id)
        // FIX: See if I can avoid this clone
        .system(SystemContentBlock::Text(system_prompt.to_string()))
        .set_messages(Some(msg))
        .inference_config(inference_parameters)
        .send()
        .await;

    let mut stream = match response {
        Ok(output) => Ok(output.stream),
        Err(e) => Err(BedrockConverseStreamError::from(
            e.as_service_error().unwrap(),
        )),
    }?;

    // A string that response the message back
    let mut output = String::new();

    // return the conversation
    let mut convo = Conversation::new(ConversationEntity::Assistant, String::new());

    // the main printing loop
    loop {
        let token = stream.recv().await;
        match token {
            Ok(Some(text)) => {
                let next = get_converse_output_text(text)?;
                print!("{}", next);
                output.push_str(&next);
                Ok(())
            }
            Ok(None) => {
                convo.content.push_str(&output);
                break;
            }
            Err(e) => Err(e
                .as_service_error()
                .map(BedrockConverseStreamError::from)
                .unwrap_or(BedrockConverseStreamError(
                    "Unknown error recieving stream".into(),
                ))),
        }?
    }

    println!();

    Ok(convo)
}
