#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::{
    debug_handler,
    http::{header, StatusCode},
};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    common::settings::Settings,
    models::{
        _entities::{mixarts, mixes},
        arts::{self, ModelVec},
        mixarts::MixArtParams,
        mixes::MixParams,
    },
    services::{
        providers::{ImageProvider, TextProvider},
        service_provider::ServiceProvider,
    },
    tasks::art_prompts::{MIX_IMAGE_PROMPT, TITLE_PROMPT},
    views,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/mix")
        .add("/", get(show_form))
        .add("/", post(create))
        .add("/:id", get(show))
}

#[derive(Debug, Deserialize)]
pub struct MixReqParams {
    art_ids: Vec<i32>,
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<mixes::Model> {
    let item = mixes::Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let ids = mixarts::Model::find_art_ids(&ctx.db, id).await?;

    views::mixes::show(&v, &item, &ids)
}

#[debug_handler]
pub async fn show_form(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let title_ids = arts::Model::find_all_title_ids(&ctx.db).await?;
    views::mixes::show_form(&v, &title_ids)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<MixReqParams>,
) -> Result<Response> {
    let settings = Settings::from_json(
        &ctx.config
            .settings
            .clone()
            .ok_or(Error::Message("Invalid settings".into()))?,
    )?;

    let arts = arts::Model::find_in(&ctx.db, params.art_ids.clone()).await?;

    let img_gen = ServiceProvider::img_service(&ImageProvider::Google, &settings.gemini_api_key);
    let text_gen = ServiceProvider::txt_service(&TextProvider::OpenAI, &settings.openai_key);

    let prompt = MIX_IMAGE_PROMPT.replace("{{PROMPTS}}", &arts.to_formatted_prompts());

    let prompt = text_gen
        .generate(&prompt)
        .await
        .map_err(|e| Error::Message(format!("Unable to gen prompt for mix: {e}")))?;

    let titles = arts.to_formatted_titles();
    let title_prompt = TITLE_PROMPT
        .replace("{{TITLES}}", &titles)
        .replace("{{DESCRIPTION}}", &prompt);

    let title = text_gen
        .generate(&title_prompt)
        .await
        .map_err(|_| Error::Message("Unable to create title for mix".into()))?;

    println!("Generating mix: {title} - {prompt}");

    let image = img_gen
        .generate(&prompt)
        .await
        .map_err(|e| Error::Message(format!("Unable to generate image: {e}")))?;

    let mix = mixes::Model::create(
        &ctx.db,
        &MixParams {
            image,
            prompt,
            title,
            model: img_gen.model_name(),
        },
    )
    .await?;

    //NOTE: what happens if just this step fails?
    //I'd get a dangling mix.
    mixarts::Model::create(
        &ctx.db,
        &MixArtParams {
            mix_id: mix.id,
            art_ids: params.art_ids,
        },
    )
    .await?;

    Ok((
        StatusCode::SEE_OTHER,
        [(header::LOCATION, format!("/mix/{}", mix.id))],
        "Redirecting to mix result...",
    )
        .into_response())
}
