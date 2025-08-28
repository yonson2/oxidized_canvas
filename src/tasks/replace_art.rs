use loco_rs::prelude::*;
use sea_orm::{EntityTrait, Set};

use crate::{
    common,
    models::_entities::arts,
    models::_entities::arts::ActiveModel as ArtActiveModel,
    services::{
        providers::{ImageProvider, TextProvider},
        service_provider::ServiceProvider,
    },
    tasks::art_prompts::{IMAGE_PROMPT, SAMPLE_PROMPTS, SAMPLE_TITLES, TITLE_PROMPT},
};

pub struct ReplaceArt;
#[async_trait]
impl Task for ReplaceArt {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "replace_art".to_string(),
            detail: "Replaces an existing art with a new AI-generated one. Usage: cargo loco task replace_art id:123"
                .to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        println!("Running `replace_art` task");

        let art_id_str = vars.cli_arg("id")?;

        let art_id = art_id_str.parse::<i32>().map_err(|e| {
            loco_rs::errors::Error::string(&format!(
                "Invalid 'id': {e}. Must be an integer. Usage: cargo loco task replace_art id:123"
            ))
        })?;

        println!("Attempting to replace art with ID: {}", art_id);

        let settings = common::settings::Settings::from_json(&ctx.config.settings.clone().ok_or_else(|| loco_rs::errors::Error::Message("Settings not found in AppContext".to_string()))?)?;

        let img_gen = ServiceProvider::img_service(
            &ImageProvider::OpenAI,
            &settings.openai_key,
        );
        let text_gen =
            ServiceProvider::txt_service(&TextProvider::Anthropic, &settings.anthropic_key);

        let art_to_replace = arts::Entity::find_by_id(art_id)
            .one(&ctx.db)
            .await? 
            .ok_or_else(|| {
                loco_rs::errors::Error::string(&format!("Art with ID {} not found", art_id))
            })?;

        println!(
            "Found art: {} - {}",
            art_to_replace.id,
            art_to_replace.title
        );

        let arts = arts::Model::find_n_latest(&ctx.db, 10).await?;
        let image_generator_prompt = match arts.len() {
            x if x > 1 => gen_img_prompt(&arts),
            _ => IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS),
        };

        println!("Generating new image prompt...");
        let new_prompt = match text_gen.generate(&image_generator_prompt).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error generating image prompt: {:?}", e);
                return Err(loco_rs::errors::Error::Message(
                    "Failed to generate image prompt".to_string(),
                ));
            }
        };
        println!("New prompt for image is: {}", new_prompt);

        println!("Generating new image...");
        let new_image_url = match img_gen.generate(&new_prompt).await {
            Ok(url) => url,
            Err(e) => {
                eprintln!("Error generating image: {:?}", e);
                return Err(loco_rs::errors::Error::Message(
                    "Failed to generate image".to_string(),
                ));
            }
        };
        println!("New image URL: {}", new_image_url);

        let title_generator_prompt = match arts.len() {
            x if x > 1 => gen_title_prompt(&new_prompt, &arts),
            _ => TITLE_PROMPT
                .replace("{{TITLES}}", SAMPLE_TITLES)
                .replace("{{DESCRIPTION}}", &new_prompt),
        };

        println!("Generating new title...");
        let new_title = match text_gen.generate(&title_generator_prompt).await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error generating title: {:?}", e);
                return Err(loco_rs::errors::Error::Message(
                    "Failed to generate title".to_string(),
                ));
            }
        };
        println!("New title: {}", new_title);

        let mut art_active_model: ArtActiveModel = art_to_replace.into();
        art_active_model.prompt = Set(new_prompt);
        art_active_model.image = Set(new_image_url);
        art_active_model.title = Set(new_title);
        art_active_model.updated_at = Set(chrono::Utc::now().into());

        println!("Updating art record in database...");
        let updated_art = art_active_model.update(&ctx.db).await?;

        println!(
            "Successfully replaced art: {} - {}",
            updated_art.id,
            updated_art.title
        );
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
