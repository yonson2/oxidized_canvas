use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ImageProvider {
    OpenAI,
    Bfl,
}

impl ImageProvider {
    pub fn random() -> Self {
        if fastrand::bool() {
            ImageProvider::OpenAI
        } else {
            ImageProvider::Bfl
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TextProvider {
    Anthropic,
}
