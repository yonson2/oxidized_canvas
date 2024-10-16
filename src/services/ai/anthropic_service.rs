use anthropic::{
    client::ClientBuilder,
    types::{ContentBlock, Message, MessagesRequestBuilder, Role},
};
use axum::async_trait;

use crate::errors::Error;

use super::traits::TextGenerator;

pub struct AnthropicService {
    api_key: String,
}

impl AnthropicService {
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl TextGenerator for AnthropicService {
    async fn generate(&self, prompt: &str) -> Result<String, crate::errors::Error> {
        // yahyah, generated every invocation.
        let client = ClientBuilder::default()
            .api_key(self.api_key.clone())
            .build()?;

        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: prompt.into(),
            }],
        }];

        let messages_request = MessagesRequestBuilder::default()
            .messages(messages.clone())
            .model("claude-3-5-sonnet-20240620".to_string())
            .max_tokens(8192_usize)
            .build()?;

        // Send a completion request.
        let messages_response = client.messages(messages_request).await?;
        let text = messages_response
            .content
            .into_iter()
            .filter_map(|v| match v {
                ContentBlock::Text { text } => Some(text),
                ContentBlock::Image { .. } => None,
            })
            .collect::<String>();
        Ok(text)
    }
}

impl From<anthropic::client::ClientBuilderError> for Error {
    fn from(value: anthropic::client::ClientBuilderError) -> Self {
        Self::AIError(format!("Error building client: {value}"))
    }
}

impl From<anthropic::error::AnthropicError> for Error {
    fn from(value: anthropic::error::AnthropicError) -> Self {
        Self::AIError(format!("Error querying claude: {value}"))
    }
}
