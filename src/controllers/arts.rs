#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use axum::{extract::Form, response::Redirect};
use loco_rs::prelude::*;
use sea_orm::{sea_query::Order, QueryOrder};
use serde::{Deserialize, Serialize};

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
    let item = load_item(&ctx, id).await?;
    views::arts::show(&v, &item)
}

#[debug_handler]
pub async fn show_latest(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Model::find_latest(&ctx.db).await?;
    views::arts::show(&v, &item)
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

pub fn index() -> Routes {
    Routes::new()
        .add("/", get(show_latest))
        .add("/:id", get(show))
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
