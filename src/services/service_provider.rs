use super::ai::{
    openrouter_service::OpenRouterService,
    traits::{ImageGenerator, TextGenerator},
};
use crate::{common::settings::Settings, errors::Error};

pub struct ServiceProvider {}

fn is_configured(key: &str) -> bool {
    let key = key.trim();
    !key.is_empty() && key != "openrouter_api_key_goes_here"
}

fn draw(pool: &[String]) -> Option<&str> {
    if pool.is_empty() {
        return None;
    }
    pool.get(fastrand::usize(..pool.len())).map(String::as_str)
}

fn draw_service(pool: &[String], kind: &str, api_key: &str) -> Result<OpenRouterService, Error> {
    let model = draw(pool)
        .ok_or_else(|| Error::AIError(format!("No {kind} models configured")))?;
    if !is_configured(api_key) {
        return Err(Error::AIError(
            "OpenRouter API key is not configured".to_string(),
        ));
    }
    OpenRouterService::new(api_key, model)
}

impl ServiceProvider {
    /// `random_img_service` draws a Model from the image Model Pool and returns
    /// a generator bound to it.
    ///
    /// # Errors
    ///
    /// If the image Model Pool is empty, or the OpenRouter key is not configured.
    pub fn random_img_service(settings: &Settings) -> Result<Box<dyn ImageGenerator + Send>, Error> {
        Ok(Box::new(draw_service(
            &settings.image_models,
            "image",
            &settings.openrouter_api_key,
        )?))
    }

    /// `random_txt_service` draws a Model from the text Model Pool and returns
    /// a generator bound to it.
    ///
    /// # Errors
    ///
    /// If the text Model Pool is empty, or the OpenRouter key is not configured.
    pub fn random_txt_service(settings: &Settings) -> Result<Box<dyn TextGenerator + Send>, Error> {
        Ok(Box::new(draw_service(
            &settings.text_models,
            "text",
            &settings.openrouter_api_key,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(text_models: Vec<String>, image_models: Vec<String>) -> Settings {
        Settings {
            openrouter_api_key: "sk-or-test-key".to_string(),
            text_models,
            image_models,
            ..Default::default()
        }
    }

    #[test]
    fn random_img_service_draws_the_only_image_model() {
        let s = settings(
            vec!["text/only-model".to_string()],
            vec!["image/only-model".to_string()],
        );
        let generator = ServiceProvider::random_img_service(&s).unwrap();
        assert_eq!(generator.model_name(), "OpenRouter: image/only-model");
    }

    #[test]
    fn random_txt_service_draws_the_only_text_model() {
        let s = settings(
            vec!["text/only-model".to_string()],
            vec!["image/only-model".to_string()],
        );
        let generator = ServiceProvider::random_txt_service(&s).unwrap();
        assert_eq!(generator.model_name(), "OpenRouter: text/only-model");
    }

    #[test]
    fn random_img_service_draws_a_pool_member() {
        let pool = vec![
            "image/a".to_string(),
            "image/b".to_string(),
            "image/c".to_string(),
        ];
        let s = settings(vec!["text/x".to_string()], pool.clone());
        let generator = ServiceProvider::random_img_service(&s).unwrap();
        let name = generator.model_name();
        assert!(
            pool.iter().any(|m| name == format!("OpenRouter: {m}")),
            "drew {name}, expected a pool member"
        );
    }

    #[test]
    fn random_img_service_fails_when_image_pool_empty() {
        let s = settings(vec!["text/x".to_string()], vec![]);
        let err = ServiceProvider::random_img_service(&s).err().unwrap();
        assert!(err.to_string().contains("image"), "{err}");
    }

    #[test]
    fn random_txt_service_fails_when_text_pool_empty() {
        let s = settings(vec![], vec!["image/x".to_string()]);
        let err = ServiceProvider::random_txt_service(&s).err().unwrap();
        assert!(err.to_string().contains("text"), "{err}");
    }

    #[test]
    fn factories_fail_when_api_key_is_a_placeholder() {
        for bad_key in ["", "openrouter_api_key_goes_here"] {
            let mut s = settings(vec!["text/x".to_string()], vec!["image/x".to_string()]);
            s.openrouter_api_key = bad_key.to_string();
            assert!(
                ServiceProvider::random_img_service(&s).is_err(),
                "img factory accepted key {bad_key:?}"
            );
            assert!(
                ServiceProvider::random_txt_service(&s).is_err(),
                "txt factory accepted key {bad_key:?}"
            );
        }
    }
}
