#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use axum::extract::Query;
use axum::http::{header, StatusCode};
use loco_rs::model::query::PaginationQuery;
use loco_rs::prelude::*;
use sitemap_rs::image::Image;
use sitemap_rs::url::{ChangeFrequency, Url};
use sitemap_rs::url_set::UrlSet;

use crate::views::arts::PaginationResponse;
use crate::{
    models::_entities::arts::{Entity, Model},
    views,
};

use super::utils::ExtractId;

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
pub async fn show_latest(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Model::find_latest(&ctx.db).await?;
    views::arts::show(&v, &item, true)
}

#[debug_handler]
pub async fn show_infinite(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let items = Model::find_all_latest(
        &ctx.db,
        &PaginationQuery {
            page: 1,
            page_size: 5,
        },
    )
    .await?;
    views::arts::show_infinite(&v, &items.page)
}

#[debug_handler]
pub async fn infinite_json(
    State(ctx): State<AppContext>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Response> {
    let items = Model::find_all_latest(&ctx.db, &pagination).await?;
    format::json(PaginationResponse::response(items, &pagination))
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
        .add("/infinite", get(show_infinite))
        .add("/infinite.json", get(infinite_json))
        .add("/img/:id", get(serve_image))
        .add("/sitemap.xml", get(sitemap))
}
