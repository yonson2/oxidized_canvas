use loco_rs::errors::Error;
use loco_rs::prelude::*;

use crate::{
    common,
    models::{_entities::arts, arts::ArtParams},
    services::service_provider::ServiceProvider,
    tasks::art_prompts::{IMAGE_PROMPT, SAMPLE_PROMPTS, SAMPLE_TITLES, TITLE_PROMPT},
};

pub struct CreateArt;
#[async_trait]
impl Task for CreateArt {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "create_art".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        let settings = common::settings::Settings::from_json(
            &ctx.config
                .settings
                .clone()
                .ok_or(Error::Message("Invalid settings".into()))?,
        )?;

        let img_gen = ServiceProvider::random_img_service(&settings);
        let text_gen = ServiceProvider::random_txt_service(&settings);

        let random_arts = arts::Model::find_n_random(&ctx.db, 5).await?;
        let latest_arts = arts::Model::find_n_latest(&ctx.db, 5).await?;
        let image_generator_prompt = match (random_arts.len(), latest_arts.len()) {
            (0, 0) => IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS),
            _ => gen_img_prompt(&random_arts, &latest_arts),
        };

        let prompt = text_gen
            .generate(&image_generator_prompt)
            .await
            .map_err(|e| Error::Message(format!("Unable to generate prompt for image: {e}")))?;

        println!("Prompt for image is: {prompt}");

        let title_generator_prompt = match (random_arts.len(), latest_arts.len()) {
            (0, 0) => TITLE_PROMPT
                .replace("{{TITLES}}", SAMPLE_TITLES)
                .replace("{{DESCRIPTION}}", &prompt),
            _ => gen_title_prompt(&prompt, &random_arts, &latest_arts),
        };

        let title = text_gen
            .generate(&title_generator_prompt)
            .await
            .map_err(|e| Error::Message(format!("Unable to generate title: {e}")))?;

        let image = img_gen
            .generate(&prompt)
            .await
            .map_err(|e| Error::Message(format!("Unable to generate image: {e}")))?;

        let art = arts::Model::create(
            &ctx.db,
            &ArtParams {
                image,
                prompt,
                title,
                model: Some(img_gen.model_name()),
            },
        )
        .await?;

        println!("Created art: {} - {}", art.id, art.title);
        Ok(())
    }
}

fn gen_title_prompt(
    desc: &str,
    random_arts: &[arts::Model],
    latest_arts: &[arts::Model],
) -> String {
    let mut all_titles = Vec::new();

    // Collect titles from both random and latest arts
    for art in random_arts.iter().chain(latest_arts.iter()) {
        all_titles.push(art.title.clone());
    }

    let titles = all_titles.join(", ");

    TITLE_PROMPT
        .replace("{{TITLES}}", &titles)
        .replace("{{DESCRIPTION}}", desc)
}

fn gen_img_prompt(random_arts: &[arts::Model], latest_arts: &[arts::Model]) -> String {
    let mut prompts = Vec::new();

    // Add random arts for inspiration
    if !random_arts.is_empty() {
        prompts.push("Previous prompts for inspiration (try to vary from these):".to_string());
        for (i, art) in random_arts.iter().enumerate() {
            prompts.push(format!(" - inspiration {}: {}", i + 1, art.prompt.clone()));
        }
    }

    // Add latest arts with stronger emphasis on being different
    if !latest_arts.is_empty() {
        prompts.push(
            "\nRecent prompts to actively differentiate from (be distinctly different from these):"
                .to_string(),
        );
        for (i, art) in latest_arts.iter().enumerate() {
            prompts.push(format!(" - recent {}: {}", i + 1, art.prompt.clone()));
        }
    }

    let combined_prompts = prompts.join("\n");
    IMAGE_PROMPT.replace("{{PROMPTS}}", &combined_prompts)
}
