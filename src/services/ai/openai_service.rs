use std::io::Cursor;

use axum::async_trait;
use base64::{engine::general_purpose, Engine};
use image::{load_from_memory, ImageFormat};
use serde::{Deserialize, Serialize};

use super::traits::ImageGenerator;
use crate::errors::Error;

const OPENAI_API_ENDPOINT: &str = "https://api.openai.com/v1/images/generations";

pub struct OpenAIService {
    api_key: String,
}

impl OpenAIService {
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIImageRequestPayload<'a> {
    size: &'a str,
    quality: &'a str,
    output_format: &'a str,
    prompt: &'a str,
    model: &'a str,
}

#[derive(Debug, Deserialize)]
struct OpenAIImageResponse {
    data: Vec<OpenAIImageData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIImageData {
    b64_json: String,
}

#[async_trait]
impl ImageGenerator for OpenAIService {
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let payload = OpenAIImageRequestPayload {
            model: "gpt-image-1",
            quality: "high",
            prompt,
            size: "1024x1024",
            output_format: "webp",
        };

        let response_result = ureq::post(OPENAI_API_ENDPOINT)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(ureq::json!(payload));

        let response = match response_result {
            Ok(res) => res,
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Failed to read error body".to_string());
                return Err(Error::AIError(format!(
                    "OpenAI API request failed with status {}: {}",
                    code, error_body
                )));
            }
            Err(transport_err) => {
                return Err(Error::AIError(format!(
                    "OpenAI API transport error: {}",
                    transport_err
                )));
            }
        };

        let image_response: OpenAIImageResponse = response.into_json()?;

        let b64_json_data = image_response
            .data
            .into_iter()
            .next()
            .map(|d| d.b64_json)
            .ok_or_else(|| {
                Error::AIError("OpenAI response did not contain image data".to_string())
            })?;

        let image_bytes = general_purpose::STANDARD.decode(&b64_json_data)?;

        let webp_bytes = to_webp(&image_bytes)?;

        let base64_webp_string = to_base64(&webp_bytes);

        Ok(base64_webp_string)
    }
}

fn to_webp(image_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(image_bytes)?;
    let mut webp_buffer = Cursor::new(Vec::new());
    img.write_to(&mut webp_buffer, ImageFormat::WebP)?;
    Ok(webp_buffer.into_inner())
}

fn to_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::AIError(format!("Base64 decoding error: {}", value))
    }
}
