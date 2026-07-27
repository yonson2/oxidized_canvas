use crate::errors::Error;

impl From<openrouter_rs::error::OpenRouterError> for Error {
    fn from(value: openrouter_rs::error::OpenRouterError) -> Self {
        Self::AIError(format!("OpenRouter request failed: {value}"))
    }
}

impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::AIError(format!("Error converting file: {value}"))
    }
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::AIError(format!("Base64 decoding error: {}", value))
    }
}
