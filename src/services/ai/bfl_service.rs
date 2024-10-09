use std::{io::Cursor, time::Duration};

use async_std::task;

use axum::async_trait;
use base64::{engine::general_purpose, Engine};
use image::{load_from_memory, ImageFormat};
use serde::{Deserialize, Serialize};

use super::traits::ImageGenerator;
use crate::errors::Error;

pub struct BFLService {
    endpoint: String,
    api_key: String,
}

impl BFLService {
    pub fn new(endpoint: &str, api_key: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl ImageGenerator for BFLService {
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let req: GenerateResponse = ureq::post(&self.endpoint)
            .set("accept", "application/json")
            .set("x-key", &self.api_key)
            .send_json(ureq::json!({
                "prompt": prompt,
                "height": 1440,
                "width": 1440,
                "steps": 50,
                "prompt_upsampling": true,
                "safety_tolerance": 6
            }))?
            .into_json()?;

        let url: String;
        loop {
            task::sleep(Duration::from_millis(500)).await;

            let img: ImageResult =
                ureq::get(&format!("https://api.bfl.ml/v1/get_result?id={}", req.id))
                    .set("accept", "application/json")
                    .set("x-key", &self.api_key)
                    .call()?
                    .into_json()?;

            if img.status == "Ready" {
                if let Some(u) = img.result {
                    url = u.sample;
                    break;
                }
            }
        }
        // Send a GET request to download the image
        let response = ureq::get(&url).call()?;

        // Read the response body into a Vec<u8>
        let mut buffer = Vec::new();
        response.into_reader().read_to_end(&mut buffer)?;

        let img = load_from_memory(&buffer)?;
        let mut converted_img = Cursor::new(Vec::new());
        img.write_to(&mut converted_img, ImageFormat::WebP)?;

        Ok(general_purpose::STANDARD.encode(converted_img.get_ref()))
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        crate::errors::Error::AIError(format!("Error doing network request: {value}"))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        crate::errors::Error::AIError(format!("Error loading file to memory: {value}"))
    }
}

impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        crate::errors::Error::AIError(format!("Error converting file: {value}"))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct GenerateResponse {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ImageResult {
    id: String,
    status: String,
    result: Option<Image>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Image {
    sample: String,
    prompt: String,
}
