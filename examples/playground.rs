use oxidized_canvas::{
    common,
    services::{
        ai::traits::ImageGenerator, ai::traits::TextGenerator, service_provider::ServiceProvider,
    },
};

#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use oxidized_canvas::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;
    let settings = common::settings::Settings::from_json(&ctx.config.settings.ok_or(0).unwrap())?;

    // let active_model: articles::ActiveModel = ActiveModel {
    //     title: Set(Some("how to build apps in 3 steps".to_string())),
    //     content: Set(Some("use Loco: https://loco.rs".to_string())),
    //     ..Default::default()
    // };
    // active_model.insert(&ctx.db).await.unwrap();

    // let res = articles::Entity::find().all(&ctx.db).await.unwrap();
    // println!("{:?}", res);
    println!("welcome to playground. edit me at `examples/playground.rs`");

    let ai = ServiceProvider::txt_service(&settings.anthropic_key);
    match ai.generate("What is 3 + 2?").await {
        Ok(v) => println!("WOW, {v}"),
        Err(e) => println!("Error {e}"),
    }

    let img_ai = ServiceProvider::img_service(&settings.openai_key);

    match img_ai
        .generate("A futuristic cityscape with flying cars, neon lights, and towering skyscrapers, digital art") // New prompt for image generation
        .await
    {
        Ok(base64_image) => {
            println!("Successfully generated image (base64 WebP).");
            // To save the image, you would typically decode the base64 string
            // and write the bytes to a file. For example:
            use base64::{engine::general_purpose, Engine};
            use std::fs::File;
            use std::io::Write;
            let image_bytes = general_purpose::STANDARD.decode(&base64_image).unwrap();
            let mut file = File::create("generated_image.webp").unwrap();
            file.write_all(&image_bytes).unwrap();
            println!("Image saved to generated_image.webp");
            // For playground, just printing a snippet.
            if base64_image.len() > 100 {
                println!("Image data (first 100 chars): {}...", &base64_image[..100]);
            } else {
                println!("Image data: {}", base64_image);
            }
        }
        Err(e) => println!("Error generating image: {e}"),
    }

    Ok(())
}
