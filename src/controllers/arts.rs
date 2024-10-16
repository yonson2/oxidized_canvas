#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use axum::http::{header, StatusCode};
use axum::{extract::Form, response::Redirect};
use loco_rs::prelude::*;
use regex::Regex;
use sea_orm::{sea_query::Order, QueryOrder};
use serde::{Deserialize, Serialize};
use sitemap_rs::image::Image;
use sitemap_rs::url::{ChangeFrequency, Url};
use sitemap_rs::url_set::UrlSet;

use crate::{
    models::_entities::arts::{ActiveModel, Column, Entity, Model},
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub image: String,
    pub prompt: String,
    pub title: String,
    pub uuid: Uuid,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.image = Set(self.image.clone());
        item.prompt = Set(self.prompt.clone());
        item.title = Set(self.title.clone());
        item.uuid = Set(self.uuid);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .order_by(Column::Id, Order::Desc)
        .all(&ctx.db)
        .await?;
    views::arts::list(&v, &item)
}

#[debug_handler]
pub async fn new(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    views::arts::create(&v)
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    item.update(&ctx.db).await?;
    Ok(Redirect::to("../arts"))
}

#[debug_handler]
pub async fn edit(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::arts::edit(&v, &item)
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
pub async fn add(State(ctx): State<AppContext>, Form(params): Form<Params>) -> Result<Redirect> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    item.insert(&ctx.db).await?;
    Ok(Redirect::to("arts"))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
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
pub fn index() -> Routes {
    Routes::new()
        .add("/", get(show_latest))
        .add("/:id", get(show))
        .add("/img/:id", get(serve_image))
        .add("/sitemap.xml", get(sitemap))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("arts/")
        .add("/", get(show_latest))
        // .add("/", get(list))
        .add("/", post(add))
        .add("new", get(new))
        .add(":id", get(show))
        .add(":id/edit", get(edit))
        .add(":id", post(update))
        .add(":id", delete(remove))
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
