use loco_rs::prelude::*;

use crate::{
    common,
    models::{_entities::arts, arts::ArtParams},
    services::{
        ai::traits::{ImageGenerator, TextGenerator},
        service_provider::ServiceProvider,
    },
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
        let settings =
            common::settings::Settings::from_json(&ctx.config.settings.clone().ok_or(0).unwrap())?;

        let img_gen = ServiceProvider::img_service(&settings.openai_key);
        let text_gen = ServiceProvider::txt_service(&settings.anthropic_key);

        let arts = arts::Model::find_n_latest(&ctx.db, 10).await?;
        let image_generator_prompt = match arts.len() {
            x if x > 1 => gen_img_prompt(&arts),
            _ => IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS),
        };

        let Ok(prompt) = text_gen.generate(&image_generator_prompt).await else {
            return Err(loco_rs::errors::Error::Message("text_gen 1".to_string()));
        };
        println!("Prompt for image is: {prompt}");

        let title_generator_prompt = match arts.len() {
            x if x > 1 => gen_title_prompt(&prompt, &arts),
            _ => TITLE_PROMPT
                .replace("{{TITLES}}", SAMPLE_TITLES)
                .replace("{{DESCRIPTION}}", &prompt),
        };

        let (title, image) = (
            match text_gen.generate(&title_generator_prompt).await {
                Ok(t) => t,
                Err(_) => return Err(loco_rs::errors::Error::Message("text_gen 2".to_string())),
            },
            match img_gen.generate(&prompt).await {
                Ok(i) => i,
                Err(e) => {
                    println!("ERROR: {e}");
                    return Err(loco_rs::errors::Error::Message("img_gen 1".to_string()));
                }
            },
        );

        let art = arts::Model::create(
            &ctx.db,
            &ArtParams {
                image,
                prompt,
                title,
            },
        )
        .await?;

        println!("Created art: {} - {}", art.id, art.title);
        Ok(())
    }
}

fn gen_title_prompt(desc: &str, arts: &[arts::Model]) -> String {
    let titles = arts
        .iter()
        .map(|a| a.title.clone())
        .collect::<Vec<String>>()
        .join(", ");

    TITLE_PROMPT
        .replace("{{TITLES}}", &titles)
        .replace("{{DESCRIPTION}}", desc)
}

fn gen_img_prompt(arts: &[arts::Model]) -> String {
    let prompts = arts
        .iter()
        .enumerate()
        .map(|(i, a)| format!(" - prompt {i}: {}", a.title.clone()))
        .collect::<Vec<String>>()
        .join("\n");
    IMAGE_PROMPT.replace("{{PROMPTS}}", &prompts)
}

