use loco_rs::cli::playground;
use oxidized_canvas::{
    app::App, common, services::service_provider::ServiceProvider,
};

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;
    let settings = common::settings::Settings::from_json(&ctx.config.settings.ok_or(0).unwrap())?;

    println!("---OpenRouter Example---");

    // Text Generation
    println!("\nGenerating text...");
    let txt_ai = ServiceProvider::random_txt_service(&settings)
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
    println!("Using model: {}", txt_ai.model_name());
    match txt_ai.generate("What is the meaning of life?").await {
        Ok(v) => println!("Generated text: {v}"),
        Err(e) => println!("Error generating text: {e}"),
    }

    // Image Generation
    println!("\nGenerating image...");
    let img_ai = ServiceProvider::random_img_service(&settings)
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
    println!("Using model: {}", img_ai.model_name());
    match img_ai
        .generate("A photorealistic image of a cat programming on a laptop")
        .await
    {
        Ok(base64_image) => {
            println!("Successfully generated image.");
            save_image(&base64_image, "openrouter_image.webp");
        }
        Err(e) => println!("Error generating image: {e}"),
    }

    Ok(())
}

fn save_image(base64_string: &str, filename: &str) {
    use base64::{Engine, engine::general_purpose};
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
