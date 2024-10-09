use axum::async_trait;

use crate::errors::Error;

#[async_trait]
pub trait ImageGenerator {
    /// generate takes a prompt and returns a Base64 encoding of the image in WebP format.
    async fn generate(&self, prompt: &str) -> Result<String, Error>;
}

//TODO: change name? merge traits? leave as is? (TO-THINK)
#[async_trait]
pub trait TextGenerator {
    /// generate takes a prompt and returns a Base64 encoding of the image in WebP format.
    async fn generate(&self, prompt: &str) -> Result<String, Error>;
}
