#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use axum::http::{header, StatusCode};
use loco_rs::prelude::*;
use regex::Regex;
use serde::Deserialize;
use sitemap_rs::image::Image;
use sitemap_rs::url::{ChangeFrequency, Url};
use sitemap_rs::url_set::UrlSet;

use crate::common::settings::Settings;
use crate::models::arts::ModelVec;
use crate::services::providers::TextProvider;
use crate::services::service_provider::ServiceProvider;
use crate::tasks::art_prompts::{MIX_IMAGE_PROMPT, TITLE_PROMPT};
use crate::{
    models::_entities::arts::{Entity, Model},
    views,
};

#[derive(Debug, Deserialize)]
pub struct MixParams {
    art_ids: Vec<i32>,
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let latest_id = Model::find_latest_id(&ctx.db).await?;
    let item = load_item(&ctx, id).await?;
    let latest = latest_id == item.id;

    views::arts::show(&v, &item, latest)
}

#[debug_handler]
pub async fn show_mix(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let title_ids = Model::find_all_title_ids(&ctx.db).await?;
    views::arts::show_mix(&v, &title_ids)
}

#[debug_handler]
pub async fn create_mix(
    State(ctx): State<AppContext>,
    Json(params): Json<MixParams>,
) -> Result<Response> {
    let settings = Settings::from_json(
        &ctx.config
            .settings
            .clone()
            .ok_or(Error::Message("Invalid settings".into()))?,
    )?;

    let arts = Model::find_in(&ctx.db, params.art_ids).await?;

    // let img_gen = ServiceProvider::random_img_service(&settings);
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

    //TODO:
    // need to generate new models and run migrations for mixes.
    // need to insert a new mix.
    // need to add a route to view the mixes
    // need to redirect to the mixes.
    // move mix routes to mix controller

    println!("{title} - {prompt}");

    Ok((
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/")],
        "Redirecting to mix result...",
    )
        .into_response())
}

#[debug_handler]
pub async fn show_latest(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Model::find_latest(&ctx.db).await?;
    views::arts::show(&v, &item, true)
}

#[debug_handler]
pub async fn serve_image(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    //TODO: support png too.
    let (id, _format) = id.extract_id().ok_or_else(|| Error::NotFound)?;
    let bytes = Model::find_img_slice_by_id(&ctx.db, id).await?;

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

#[debug_handler]
/// `sitemap` builds a sitemap.xml for our site
/// # Panics
///
/// Panics because I haven't handled the `TODO` yet.
pub async fn sitemap(State(ctx): State<AppContext>) -> Result<Response> {
    let ids = Model::find_ids(&ctx.db).await?;
    //TODO: move to settings
    let base_url = "https://imaginarygallery.net/";

    //TODO: remove unwraps, don't want to be bothered converting
    // this code has been working for a year.
    let mut urls: Vec<Url> = vec![
        Url::builder(String::from(base_url))
            .change_frequency(ChangeFrequency::Daily)
            .priority(1.0)
            .build()
            .expect("Valid sitemap config"),
        Url::builder(format!("{}{}", base_url, "about"))
            .change_frequency(ChangeFrequency::Yearly)
            .priority(0.9)
            .build()
            .expect("Valid sitemap config"),
    ];

    for id in ids {
        urls.push(
            Url::builder(format!("{base_url}{id}"))
                .images(vec![Image::new(format!("{base_url}img/{id}.webp"))])
                .priority(0.8)
                .change_frequency(ChangeFrequency::Yearly)
                .build()
                .expect("Valid sitemap url"),
        );
    }
    let url_set: UrlSet = UrlSet::new(urls).expect("valid urlset");
    let mut buf = Vec::<u8>::new();
    url_set.write(&mut buf).expect("write urlset to buffer");

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], buf).into_response())
}
pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(show_latest))
        .add("/:id", get(show))
        .add("/mix", get(show_mix).post(create_mix))
        .add("/img/:id", get(serve_image))
        .add("/sitemap.xml", get(sitemap))
}

trait ExtractId {
    fn extract_id(&self) -> Option<(u32, ImageFormat)>;
}

impl ExtractId for String {
    fn extract_id(&self) -> Option<(u32, ImageFormat)> {
        let re = Regex::new(r"^(\d+)(\.png|.webp)$").unwrap();
        let captures = re.captures(self)?;

        let id = captures.get(1)?.as_str().parse::<u32>().ok()?;
        let format = match captures.get(2)?.as_str() {
            ".png" => Some(ImageFormat::Png),
            ".webp" => Some(ImageFormat::WebP),
            _ => None,
        }?;

        Some((id, format))
    }
}

enum ImageFormat {
    Png,
    WebP,
}
