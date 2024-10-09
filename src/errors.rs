use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while talking to Ai: {0}")]
    AIError(String),
}
