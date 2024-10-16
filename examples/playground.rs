use oxidized_canvas::{
    common,
    services::{
        /* ai::traits::ImageGenerator */ ai::traits::TextGenerator,
        service_provider::ServiceProvider,
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
    println!("{}", settings.anthropic_key);

    // let ai = ServiceProvider::img_service(&settings.bfl_api_key);
    let ai = ServiceProvider::txt_service(&settings.anthropic_key);

    match ai.generate("What is 3 + 2?").await {
        Ok(v) => println!("WOW, {v}"),
        Err(e) => println!("Error {e}"),
    }

    // let bfl = BFLService::new(
    //     "https://api.bfl.ml/v1/flux-pro",
    //     "d368fae6-7eaa-4460-a034-751e0a6fba7d",
    // );
    //
    // match ai
    //     .generate("a rotating dog eating hot dogs in the style of van gogh, macro lens,  4k, hd")
    //     .await
    // {
    //     Ok(v) => println!("WOW, {v}"),
    //     Err(e) => println!("Error {e}"),
    // }

    Ok(())
}
