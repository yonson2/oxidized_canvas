use std::io::Cursor;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use image::{ImageFormat, load_from_memory};
use openrouter_rs::{
    OpenRouterClient,
    api::{
        chat::{ChatCompletionRequest, Message},
        images::ImageGenerationRequest,
    },
    types::Role,
};

use super::traits::{ImageGenerator, TextGenerator};
use crate::errors::Error;

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// `OpenRouterService` generates text and images through the single Provider
/// (OpenRouter), bound to one Model drawn from a Model Pool.
pub struct OpenRouterService {
    client: OpenRouterClient,
    model: String,
}

impl OpenRouterService {
    /// # Errors
    ///
    /// If the OpenRouter client cannot be built.
    pub fn new(api_key: &str, model: &str) -> Result<Self, Error> {
        Self::with_base_url(api_key, model, OPENROUTER_BASE_URL)
    }

    fn with_base_url(api_key: &str, model: &str, base_url: &str) -> Result<Self, Error> {
        let client = OpenRouterClient::builder()
            .api_key(api_key)
            .base_url(base_url)
            .build()
            .map_err(|e| Error::AIError(format!("Error building OpenRouter client: {e}")))?;
        Ok(Self {
            client,
            model: model.to_string(),
        })
    }

    fn model_label(&self) -> String {
        format!("OpenRouter: {}", self.model)
    }
}

#[async_trait]
impl TextGenerator for OpenRouterService {
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let request = ChatCompletionRequest::builder()
            .model(&self.model)
            .messages(vec![Message::new(Role::User, prompt)])
            .build()?;

        let response = self.client.chat().create(&request).await?;

        let text = response
            .choices
            .first()
            .and_then(|choice| choice.content())
            .ok_or_else(|| {
                Error::AIError("OpenRouter response did not contain text data".to_string())
            })?;

        Ok(text.to_string())
    }

    fn model_name(&self) -> String {
        self.model_label()
    }
}

#[async_trait]
impl ImageGenerator for OpenRouterService {
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let request = ImageGenerationRequest::builder()
            .model(&self.model)
            .prompt(prompt)
            .aspect_ratio("1:1")
            .build()?;

        let response = self.client.images().create(&request).await?;

        let b64_json = response
            .data
            .first()
            .map(|image| &image.b64_json)
            .ok_or_else(|| {
                Error::AIError("OpenRouter response did not contain image data".to_string())
            })?;

        let image_bytes = general_purpose::STANDARD.decode(b64_json)?;
        let webp_bytes = to_webp(&image_bytes)?;
        Ok(general_purpose::STANDARD.encode(&webp_bytes))
    }

    fn model_name(&self) -> String {
        self.model_label()
    }
}

/// `to_webp` takes in a slice of bytes of an image and converts it to `.webp`
fn to_webp(image_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(image_bytes)?;
    let mut webp_buffer = Cursor::new(Vec::new());
    img.write_to(&mut webp_buffer, ImageFormat::WebP)?;
    Ok(webp_buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    fn chat_response(content: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "gen-1",
            "created": 1753710000,
            "model": "text/model",
            "object": "chat.completion",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": content}}]
        })
    }

    #[tokio::test]
    async fn text_generate_sends_model_and_prompt_and_returns_content() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer sk-or-test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(chat_response("a gallery of rust")))
            .mount(&server)
            .await;

        let svc = OpenRouterService::with_base_url("sk-or-test-key", "text/model", &server.uri())
            .unwrap();
        let text = TextGenerator::generate(&svc, "paint me a prompt")
            .await
            .unwrap();
        assert_eq!(text, "a gallery of rust");

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body = String::from_utf8(requests[0].body.clone()).unwrap();
        assert!(body.contains("text/model"), "{body}");
        assert!(body.contains("paint me a prompt"), "{body}");
    }

    #[tokio::test]
    async fn text_generate_fails_when_response_has_no_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "gen-2",
                "created": 1753710000,
                "model": "text/model",
                "object": "chat.completion",
                "choices": []
            })))
            .mount(&server)
            .await;

        let svc = OpenRouterService::with_base_url("sk-or-test-key", "text/model", &server.uri())
            .unwrap();
        let err = TextGenerator::generate(&svc, "prompt").await.err().unwrap();
        assert!(err.to_string().contains("text"), "{err}");
    }

    #[tokio::test]
    async fn text_generate_maps_api_errors() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(402).set_body_json(serde_json::json!({
                "error": {"message": "Insufficient credits", "code": 402}
            })))
            .mount(&server)
            .await;

        let svc = OpenRouterService::with_base_url("sk-or-test-key", "text/model", &server.uri())
            .unwrap();
        let err = TextGenerator::generate(&svc, "prompt").await.err().unwrap();
        assert!(err.to_string().contains("OpenRouter"), "{err}");
    }

    fn one_pixel_png_b64() -> String {
        use base64::{Engine, engine::general_purpose};
        let mut buffer = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(1, 1, image::Rgb([255, 0, 0]))
            .write_to(&mut buffer, image::ImageFormat::Png)
            .unwrap();
        general_purpose::STANDARD.encode(buffer.into_inner())
    }

    #[tokio::test]
    async fn image_generate_returns_base64_webp() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/images"))
            .and(header("authorization", "Bearer sk-or-test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": 1753710000,
                "data": [{"b64_json": one_pixel_png_b64(), "media_type": "image/png"}]
            })))
            .mount(&server)
            .await;

        let svc = OpenRouterService::with_base_url("sk-or-test-key", "image/model", &server.uri())
            .unwrap();
        let result = ImageGenerator::generate(&svc, "a rusty canvas")
            .await
            .unwrap();

        use base64::{Engine, engine::general_purpose};
        let bytes = general_purpose::STANDARD.decode(&result).unwrap();
        assert!(bytes.starts_with(b"RIFF"), "expected WebP bytes");
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!((img.width(), img.height()), (1, 1));

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body = String::from_utf8(requests[0].body.clone()).unwrap();
        assert!(body.contains("image/model"), "{body}");
        assert!(body.contains("a rusty canvas"), "{body}");
    }

    #[tokio::test]
    async fn image_generate_fails_when_response_has_no_image() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/images"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": 1753710000,
                "data": []
            })))
            .mount(&server)
            .await;

        let svc = OpenRouterService::with_base_url("sk-or-test-key", "image/model", &server.uri())
            .unwrap();
        let err = ImageGenerator::generate(&svc, "prompt").await.err().unwrap();
        assert!(err.to_string().contains("image"), "{err}");
    }
}
