use loco_rs::prelude::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Settings {
    pub bfl_api_key: String,
    pub anthropic_key: String,
    pub openai_key: String,
    pub old_db_url: String,
}

impl Settings {
    /// `from_json` unmarshalls our config into a type checked settings struct
    /// # Errors
    ///
    /// If there's an error unmarshalling the values, the app won't start
    /// guaranteed that no bytes were read.
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value(value.clone())?)
    }
}
