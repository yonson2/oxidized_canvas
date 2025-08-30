use std::io::Cursor;

use axum::async_trait;
use base64::{engine::general_purpose, Engine};
use image::{load_from_memory, ImageFormat};
use serde::Deserialize;

use super::traits::{ImageGenerator, TextGenerator};
use crate::errors::Error;

const GOOGLE_API_TEXT_ENDPOINT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent";
const GOOGLE_API_IMAGE_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-image-preview:generateContent";

pub struct GoogleService {
    api_key: String,
}

impl GoogleService {
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct GoogleResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize, Debug)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ResponsePart {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
}

#[derive(Deserialize, Debug)]
struct InlineData {
    #[serde(rename = "mimeType")]
    _mime_type: String,
    data: String,
}

#[async_trait]
impl TextGenerator for GoogleService {
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let payload = ureq::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        let response_result = ureq::post(GOOGLE_API_TEXT_ENDPOINT)
            .set("x-goog-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(payload);

        let response = match response_result {
            Ok(res) => res,
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Failed to read error body".to_string());
                return Err(Error::AIError(format!(
                    "Google API request failed with status {}: {}",
                    code, error_body
                )));
            }
            Err(transport_err) => {
                return Err(Error::AIError(format!(
                    "Google API transport error: {}",
                    transport_err
                )));
            }
        };

        let response_body: GoogleResponse = response.into_json()?;

        let text = response_body
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| match p {
                ResponsePart::Text { text } => Some(text),
                _ => None,
            })
            .ok_or_else(|| {
                Error::AIError("Google API response did not contain text data".to_string())
            })?;

        Ok(text)
    }
}

#[async_trait]
impl ImageGenerator for GoogleService {
    fn model_name(&self) -> String {
        "Google: gemini-2.5-flash-image".into()
    }
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let payload = ureq::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        let response_result = ureq::post(GOOGLE_API_IMAGE_ENDPOINT)
            .set("x-goog-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(payload);

        let response = match response_result {
            Ok(res) => res,
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Failed to read error body".to_string());
                return Err(Error::AIError(format!(
                    "Google API request failed with status {}: {}",
                    code, error_body
                )));
            }
            Err(transport_err) => {
                return Err(Error::AIError(format!(
                    "Google API transport error: {}",
                    transport_err
                )));
            }
        };

        let response_body: GoogleResponse = response.into_json()?;

        let b64_json_data = response_body
            .candidates
            .into_iter()
            .next()
            .and_then(|c| {
                c.content.parts.into_iter().find_map(|p| match p {
                    ResponsePart::InlineData { inline_data } => Some(inline_data.data),
                    _ => None,
                })
            })
            .ok_or_else(|| {
                Error::AIError("Google API response did not contain image data".to_string())
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
