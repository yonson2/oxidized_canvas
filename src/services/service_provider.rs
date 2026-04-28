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
use crate::common::settings::Settings;

pub struct ServiceProvider {}

fn is_configured(key: &str) -> bool {
    let key = key.trim();
    !key.is_empty() && !matches!(key, "change_me" | "bfl_api_key_goes_here" | "google_api_key_goes_here")
}

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

    #[must_use]
    pub fn random_img_service(settings: &Settings) -> Box<dyn ImageGenerator + Send> {
        let configured = [
            (ImageProvider::OpenAI, settings.openai_key.as_str()),
            (ImageProvider::Bfl, settings.bfl_api_key.as_str()),
            (ImageProvider::Google, settings.gemini_api_key.as_str()),
        ]
        .into_iter()
        .filter(|(_, key)| is_configured(key))
        .collect::<Vec<_>>();

        if configured.is_empty() {
            let provider = ImageProvider::random();
            let key = match provider {
                ImageProvider::OpenAI => &settings.openai_key,
                ImageProvider::Bfl => &settings.bfl_api_key,
                ImageProvider::Google => &settings.gemini_api_key,
            };
            Self::img_service(&provider, key)
        } else {
            let (provider, key) = &configured[fastrand::usize(..configured.len())];
            Self::img_service(provider, key)
        }
    }

    #[must_use]
    pub fn random_txt_service(settings: &Settings) -> Box<dyn TextGenerator + Send> {
        let configured = [
            (TextProvider::Anthropic, settings.anthropic_key.as_str()),
            (TextProvider::OpenAI, settings.openai_key.as_str()),
            (TextProvider::Google, settings.gemini_api_key.as_str()),
        ]
        .into_iter()
        .filter(|(_, key)| is_configured(key))
        .collect::<Vec<_>>();

        if configured.is_empty() {
            let provider = TextProvider::random();
            let key = match provider {
                TextProvider::Anthropic => &settings.anthropic_key,
                TextProvider::OpenAI => &settings.openai_key,
                TextProvider::Google => &settings.gemini_api_key,
            };
            Self::txt_service(&provider, key)
        } else {
            let (provider, key) = &configured[fastrand::usize(..configured.len())];
            Self::txt_service(provider, key)
        }
    }
}
