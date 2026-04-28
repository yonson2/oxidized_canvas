use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::{
    common::settings::Settings,
    models::arts::{self, ArtParams},
    services::{
        providers::{ImageProvider, TextProvider},
        service_provider::ServiceProvider,
    },
    tasks::art_prompts::{IMAGE_PROMPT, SAMPLE_PROMPTS, SAMPLE_TITLES, TITLE_PROMPT},
};

fn settings(ctx: &AppContext) -> Result<Settings> {
    Settings::from_json(
        &ctx.config
            .settings
            .clone()
            .ok_or(Error::Message("Invalid settings".into()))?,
    )
}

pub async fn create_art(ctx: &AppContext) -> Result<arts::Model> {
    let settings = settings(ctx)?;
    let img_gen = ServiceProvider::random_img_service(&settings);
    let text_gen = ServiceProvider::random_txt_service(&settings);

    let random_arts = arts::Model::find_n_random(&ctx.db, 5).await?;
    let latest_arts = arts::Model::find_n_latest(&ctx.db, 5).await?;
    let image_generator_prompt = match (random_arts.len(), latest_arts.len()) {
        (0, 0) => IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS),
        _ => gen_create_img_prompt(&random_arts, &latest_arts),
    };

    let prompt = text_gen
        .generate(&image_generator_prompt)
        .await
        .map_err(|e| Error::Message(format!("Unable to generate prompt for image: {e}")))?;

    let title_generator_prompt = match (random_arts.len(), latest_arts.len()) {
        (0, 0) => TITLE_PROMPT
            .replace("{{TITLES}}", SAMPLE_TITLES)
            .replace("{{DESCRIPTION}}", &prompt),
        _ => gen_create_title_prompt(&prompt, &random_arts, &latest_arts),
    };

    let title = text_gen
        .generate(&title_generator_prompt)
        .await
        .map_err(|e| Error::Message(format!("Unable to generate title: {e}")))?;

    let image = img_gen
        .generate(&prompt)
        .await
        .map_err(|e| Error::Message(format!("Unable to generate image: {e}")))?;

    arts::Model::create(
        &ctx.db,
        &ArtParams {
            image,
            prompt,
            title,
            model: Some(img_gen.model_name()),
        },
    )
    .await
    .map_err(Into::into)
}

pub async fn replace_art(ctx: &AppContext, art_id: i32) -> Result<arts::Model> {
    let settings = settings(ctx)?;
    let img_gen = ServiceProvider::img_service(&ImageProvider::OpenAI, &settings.openai_key);
    let text_gen = ServiceProvider::txt_service(&TextProvider::Anthropic, &settings.anthropic_key);

    let art_to_replace = arts::Entity::find_by_id(art_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::string(&format!("Art with ID {art_id} not found")))?;

    let recent_arts = arts::Model::find_n_latest(&ctx.db, 10).await?;
    let image_generator_prompt = if recent_arts.len() > 1 {
        gen_replace_img_prompt(&recent_arts)
    } else {
        IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS)
    };

    let prompt = text_gen
        .generate(&image_generator_prompt)
        .await
        .map_err(|e| Error::Message(format!("Failed to generate image prompt: {e}")))?;

    let image = img_gen
        .generate(&prompt)
        .await
        .map_err(|e| Error::Message(format!("Failed to generate image: {e}")))?;

    let title_generator_prompt = if recent_arts.len() > 1 {
        gen_replace_title_prompt(&prompt, &recent_arts)
    } else {
        TITLE_PROMPT
            .replace("{{TITLES}}", SAMPLE_TITLES)
            .replace("{{DESCRIPTION}}", &prompt)
    };

    let title = text_gen
        .generate(&title_generator_prompt)
        .await
        .map_err(|e| Error::Message(format!("Failed to generate title: {e}")))?;

    let mut art_active_model: arts::ActiveModel = art_to_replace.into();
    art_active_model.prompt = Set(prompt);
    art_active_model.image = Set(image);
    art_active_model.title = Set(title);
    art_active_model.model = Set(Some(img_gen.model_name()));
    art_active_model.updated_at = Set(chrono::Utc::now().into());

    art_active_model.update(&ctx.db).await.map_err(Into::into)
}

fn gen_create_title_prompt(
    desc: &str,
    random_arts: &[arts::Model],
    latest_arts: &[arts::Model],
) -> String {
    let titles = random_arts
        .iter()
        .chain(latest_arts.iter())
        .map(|art| art.title.clone())
        .collect::<Vec<String>>()
        .join(", ");

    TITLE_PROMPT
        .replace("{{TITLES}}", &titles)
        .replace("{{DESCRIPTION}}", desc)
}

fn gen_create_img_prompt(random_arts: &[arts::Model], latest_arts: &[arts::Model]) -> String {
    let mut prompts = Vec::new();

    if !random_arts.is_empty() {
        prompts.push("Previous prompts for inspiration (try to vary from these):".to_string());
        prompts.extend(
            random_arts
                .iter()
                .enumerate()
                .map(|(i, art)| format!(" - inspiration {}: {}", i + 1, art.prompt.clone())),
        );
    }

    if !latest_arts.is_empty() {
        prompts.push(
            "\nRecent prompts to actively differentiate from (be distinctly different from these):"
                .to_string(),
        );
        prompts.extend(
            latest_arts
                .iter()
                .enumerate()
                .map(|(i, art)| format!(" - recent {}: {}", i + 1, art.prompt.clone())),
        );
    }

    IMAGE_PROMPT.replace("{{PROMPTS}}", &prompts.join("\n"))
}

fn gen_replace_title_prompt(desc: &str, arts: &[arts::Model]) -> String {
    let titles = arts
        .iter()
        .map(|art| art.title.clone())
        .collect::<Vec<String>>()
        .join(", ");

    TITLE_PROMPT
        .replace("{{TITLES}}", &titles)
        .replace("{{DESCRIPTION}}", desc)
}

fn gen_replace_img_prompt(arts: &[arts::Model]) -> String {
    let prompts = arts
        .iter()
        .enumerate()
        .map(|(i, art)| format!(" - prompt {}: {}", i + 1, art.prompt.clone()))
        .collect::<Vec<String>>()
        .join("\n");

    IMAGE_PROMPT.replace("{{PROMPTS}}", &prompts)
}
