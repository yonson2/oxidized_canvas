use oxidized_canvas::{
    common,
    services::{providers::TextProvider, service_provider::ServiceProvider},
};

use loco_rs::cli::playground;
use oxidized_canvas::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;
    let settings = common::settings::Settings::from_json(&ctx.config.settings.ok_or(0).unwrap())?;

    println!("---Anthropic Example---");

    println!("\nGenerating text with Anthropic...");
    let txt_ai = ServiceProvider::txt_service(&TextProvider::Anthropic, &settings.anthropic_key);
    match txt_ai
        .generate("Write a short poem about Rust programming.")
        .await
    {
        Ok(v) => println!("Generated text: {v}"),
        Err(e) => println!("Error generating text: {e}"),
    }

    Ok(())
}
