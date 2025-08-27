use axum::async_trait;

use crate::errors::Error;

#[async_trait]
pub trait ImageGenerator: Send {
    /// generate takes a prompt and returns a Base64 encoding of the image in WebP format.
    async fn generate(&self, prompt: &str) -> Result<String, Error>;
}

//TODO: change name? merge traits? leave as is? (TO-THINK)
#[async_trait]
pub trait TextGenerator: Send {
    /// generate takes a prompt and returns a text response from AI.
    async fn generate(&self, prompt: &str) -> Result<String, Error>;
}
