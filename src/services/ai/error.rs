use crate::errors::Error;

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self::AIError(format!("Error doing network request: {value}"))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::AIError(format!("Error loading file to memory: {value}"))
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
