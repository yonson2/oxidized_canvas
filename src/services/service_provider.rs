use super::{
    ai::{
        anthropic_service::AnthropicService,
        bfl_service::BFLService,
        google_service::GoogleService,
        openai_service::OpenAIService,
        traits::{ImageGenerator, TextGenerator},
    },
    providers::{ImageProvider, TextProvider},
};

pub struct ServiceProvider {}

impl ServiceProvider {
    /// `img_service` takes in instructions and spits out base64 encoded images.
    #[must_use]
    pub fn img_service(provider: &ImageProvider, key: &str) -> Box<dyn ImageGenerator + Send> {
        match provider {
            ImageProvider::OpenAI => Box::new(OpenAIService::new(key)),
            ImageProvider::Bfl => Box::new(BFLService::new(key)),
            ImageProvider::Google => Box::new(GoogleService::new(key)),
        }
    }

    #[must_use]
    pub fn txt_service(provider: &TextProvider, key: &str) -> Box<dyn TextGenerator + Send> {
        match provider {
            TextProvider::Anthropic => Box::new(AnthropicService::new(key)),
            TextProvider::OpenAI => Box::new(OpenAIService::new(key)),
            TextProvider::Google => Box::new(GoogleService::new(key)),
        }
    }
}
