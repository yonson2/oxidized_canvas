use super::ai::{
    anthropic_service::AnthropicService,
    bfl_service::BFLService,
    traits::{ImageGenerator, TextGenerator},
};

pub struct ServiceProvider {}

impl ServiceProvider {
    /// `img_service` takes in instructions and spits out base64 encoded images.
    pub fn img_service(key: &str) -> impl ImageGenerator {
        BFLService::new("https://api.bfl.ml/v1/flux-pro", key)
    }

    pub fn txt_service(key: &str) -> impl TextGenerator {
        AnthropicService::new(key)
    }
}
