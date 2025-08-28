use oxidized_canvas::{
    common,
    services::{providers::ImageProvider, service_provider::ServiceProvider},
};

use loco_rs::cli::playground;
use oxidized_canvas::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;
    let settings = common::settings::Settings::from_json(&ctx.config.settings.ok_or(0).unwrap())?;

    println!("---BFL Example---");

    // Image Generation
    println!("\nGenerating image with BFL...");
    let img_ai = ServiceProvider::img_service(&ImageProvider::Bfl, &settings.bfl_api_key);

    match img_ai
        .generate("A beautiful landscape painting of a mountain range at sunset")
        .await
    {
        Ok(base64_image) => {
            println!("Successfully generated image.");
            save_image(&base64_image, "bfl_image.webp");
        }
        Err(e) => println!("Error generating image: {e}"),
    }

    Ok(())
}

fn save_image(base64_string: &str, filename: &str) {
    use base64::{engine::general_purpose, Engine};
    use std::fs::File;
    use std::io::Write;

    match general_purpose::STANDARD.decode(base64_string) {
        Ok(image_bytes) => {
            let mut file = File::create(filename).expect("Failed to create file");
            file.write_all(&image_bytes)
                .expect("Failed to write to file");
            println!("Image saved to {}", filename);
        }
        Err(e) => {
            println!("Error decoding base64 string: {}", e);
        }
    }
}
