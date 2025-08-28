use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ImageProvider {
    OpenAI,
    Bfl,
    Google,
}

impl ImageProvider {
    pub fn random() -> Self {
        match fastrand::u8(0..3) {
            0 => ImageProvider::OpenAI,
            1 => ImageProvider::Bfl,
            _ => ImageProvider::Google,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TextProvider {
    Anthropic,
    OpenAI,
    Google,
}

impl TextProvider {
    pub fn random() -> Self {
        match fastrand::u8(0..3) {
            0 => TextProvider::Anthropic,
            1 => TextProvider::OpenAI,
            _ => TextProvider::Google,
        }
    }
}
