use std::{io::Cursor, time::Duration};

use async_std::task;

use axum::async_trait;
use base64::{engine::general_purpose, Engine};
use image::{load_from_memory, ImageFormat};
use serde::{Deserialize, Serialize};

use super::traits::ImageGenerator;
use crate::errors::Error;

const BFL_ENDPOINT: &str = "https://api.bfl.ai/v1/flux-kontext-pro";

pub struct BFLService {
    api_key: String,
}

impl BFLService {
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl ImageGenerator for BFLService {
    fn model_name(&self) -> String {
        "BFL: Flux Kontext Pro".into()
    }
    async fn generate(&self, prompt: &str) -> Result<String, Error> {
        let req: GenerateResponse = ureq::post(BFL_ENDPOINT)
            .set("accept", "application/json")
            .set("x-key", &self.api_key)
            .send_json(ureq::json!({
                "prompt": prompt,
                "aspect_ratio": "1:1",
                "prompt_upsampling": true,
                "safety_tolerance": 6
            }))?
            .into_json()?;

        let url: String;
        loop {
            task::sleep(Duration::from_millis(500)).await;

            let img: ImageResult = ureq::get(&req.polling_url)
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

        let buffer = download_file(&url)?;
        let webp_buffer = to_webp(&buffer)?;
        Ok(to_base64(&webp_buffer))
    }
}

/// `download_file` is a little helper function that takes a url and returns
/// a Vec<u8> with its contents.
fn download_file(url: &str) -> Result<Vec<u8>, Error> {
    // Send a GET request to download the image
    let response = ureq::get(url).call()?;

    // Read the response body into a Vec<u8>
    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// `to_webp` takes in a slice of bytes of an image and converts it to `.webp`
fn to_webp(old: &[u8]) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(old)?;
    let mut converted_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut converted_img, ImageFormat::WebP)?;
    Ok(converted_img.get_ref().clone()) // moving into its own fn, need to clone.
}

fn to_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

#[derive(Debug, Deserialize, Serialize)]
struct GenerateResponse {
    id: String,
    polling_url: String,
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
