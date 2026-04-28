#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::{
    debug_handler,
    http::{StatusCode, header},
};
use loco_rs::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

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
        realtime,
        service_provider::ServiceProvider,
    },
    tasks::art_prompts::{MIX_IMAGE_PROMPT, TITLE_PROMPT},
    views,
};

use super::utils::ExtractId;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/mix")
        .add("/", get(show_form))
        .add("/", post(create))
        .add("/{id}", get(show))
        .add("/img/{id}", get(serve_image))
}

#[derive(Debug, Deserialize)]
pub struct MixReqParams {
    art_ids: Vec<i32>,
    request_id: String,
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
    let mut ids = mixarts::Model::find_art_ids(&ctx.db, id).await?;
    ids.sort();

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
    let request_id = Uuid::parse_str(&params.request_id)
        .map_err(|_| Error::Message("Invalid mix request id".into()))?;

    let result = async {
        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::new("preparing", "Collecting the selected source images..."),
        )
        .await;

        let settings = Settings::from_json(
            &ctx.config
                .settings
                .clone()
                .ok_or(Error::Message("Invalid settings".into()))?,
        )?;

        let arts = arts::Model::find_in(&ctx.db, params.art_ids.clone()).await?;

        let img_gen =
            ServiceProvider::img_service(&ImageProvider::Google, &settings.gemini_api_key);
        let text_gen = ServiceProvider::txt_service(&TextProvider::OpenAI, &settings.openai_key);

        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::new(
                "prompting",
                "Writing a shared prompt from the chosen artworks...",
            ),
        )
        .await;

        let prompt = MIX_IMAGE_PROMPT.replace("{{PROMPTS}}", &arts.to_formatted_prompts());

        let prompt = text_gen
            .generate(&prompt)
            .await
            .map_err(|e| Error::Message(format!("Unable to gen prompt for mix: {e}")))?;

        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::new(
                "titling",
                "Prompt ready. Creating a title for the new mix...",
            ),
        )
        .await;

        let titles = arts.to_formatted_titles();
        let title_prompt = TITLE_PROMPT
            .replace("{{TITLES}}", &titles)
            .replace("{{DESCRIPTION}}", &prompt);

        let title = text_gen
            .generate(&title_prompt)
            .await
            .map_err(|_| Error::Message("Unable to create title for mix".into()))?;

        println!("Generating mix: {title} - {prompt}");

        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::new(
                "rendering",
                "Title locked in. Rendering the mixed image now...",
            ),
        )
        .await;

        let image = img_gen
            .generate(&prompt)
            .await
            .map_err(|e| Error::Message(format!("Unable to generate image: {e}")))?;

        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::new(
                "saving",
                "Image finished. Saving the mix and linking the source art...",
            ),
        )
        .await;

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

        let redirect_to = format!("/mix/{}", mix.id);
        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::done("complete", "Your mix is ready. Opening it now...")
                .with_redirect_to(&redirect_to),
        )
        .await;

        Ok((
            StatusCode::SEE_OTHER,
            [(header::LOCATION, redirect_to)],
            "Redirecting to mix result...",
        )
            .into_response())
    }
    .await;

    if result.is_err() {
        realtime::emit_mix_progress(
            &request_id,
            &realtime::ProgressUpdate::failed(
                "failed",
                "The mix could not be completed. Please try again.",
            ),
        )
        .await;
    }

    result
}

#[debug_handler]
pub async fn serve_image(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    //TODO: support png too.
    let (id, _format) = id.extract_id().ok_or_else(|| Error::NotFound)?;
    let bytes = mixes::Model::find_img_slice_by_id(&ctx.db, id).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/webp"),
            (header::CACHE_CONTROL, "max-age=31536000"),
        ],
        bytes,
    )
        .into_response())
}
