use regex::Regex;

pub trait ExtractId {
    fn extract_id(&self) -> Option<(u32, ImageFormat)>;
}

impl ExtractId for String {
    fn extract_id(&self) -> Option<(u32, ImageFormat)> {
        let re = Regex::new(r"^(\d+)(\.png|.webp)$").unwrap();
        let captures = re.captures(self)?;

        let id = captures.get(1)?.as_str().parse::<u32>().ok()?;
        let format = match captures.get(2)?.as_str() {
            ".png" => Some(ImageFormat::Png),
            ".webp" => Some(ImageFormat::WebP),
            _ => None,
        }?;

        Some((id, format))
    }
}

pub enum ImageFormat {
    Png,
    WebP,
}
