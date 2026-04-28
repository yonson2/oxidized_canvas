#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]

use axum::{
    debug_handler,
    extract::Query,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use loco_rs::prelude::*;
use sea_orm::EntityTrait;
use serde::Deserialize;
use tracing::error;

use crate::{
    models::arts::{self, ArtUpdateParams},
    models::{mixarts, mixes},
    services::{art_service, backoffice_auth},
    views,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/backoffice")
        .add("", get(dashboard))
        .add("/login", get(login_form))
        .add("/login", post(login))
        .add("/logout", post(logout))
        .add("/arts", get(index))
        .add("/arts/:id", get(show))
        .add("/arts/:id", post(update))
        .add("/arts/:id/delete", post(delete))
        .add("/arts/:id/replace", post(replace))
        .add("/mixes", get(mix_index))
        .add("/mixes/:id", get(mix_show))
        .add("/mixes/:id/delete", post(mix_delete))
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginParams {
    password: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArtListQuery {
    page: Option<u64>,
    q: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MixListQuery {
    page: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArtDetailQuery {
    queued: Option<u8>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArtUpdateForm {
    title: String,
    prompt: String,
    model: String,
}

#[debug_handler]
pub async fn login_form(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
) -> Result<Response> {
    if backoffice_auth::is_authenticated(&ctx, &jar)? {
        return Ok(Redirect::to("/backoffice").into_response());
    }

    views::backoffice::login(&v, None)
}

#[debug_handler]
pub async fn login(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
    Form(params): Form<LoginParams>,
) -> Result<Response> {
    if !backoffice_auth::password_matches(&ctx, &params.password)? {
        return views::backoffice::login(
            &v,
            Some("That password did not match the configured backoffice password."),
        );
    }

    let jar = backoffice_auth::log_in(&ctx, jar)?;
    Ok((jar, Redirect::to("/backoffice")).into_response())
}

#[debug_handler]
pub async fn logout(jar: CookieJar) -> Result<Response> {
    Ok((
        backoffice_auth::log_out(jar),
        Redirect::to("/backoffice/login"),
    )
        .into_response())
}

#[debug_handler]
pub async fn dashboard(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    let stats = arts::Model::backoffice_stats(&ctx.db).await?;
    let recent_arts = arts::Model::find_n_latest(&ctx.db, 2).await?;
    let recent_mixes = mixes::Model::find_n_latest(&ctx.db, 4).await?;
    views::backoffice::dashboard(&v, &stats, &recent_arts, &recent_mixes)
}

#[debug_handler]
pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
    query: Option<Query<ArtListQuery>>,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    let query = query.map(|Query(query)| query).unwrap_or_default();
    let page =
        arts::Model::find_backoffice_page(&ctx.db, query.page.unwrap_or(1), query.q.as_deref())
            .await?;
    views::backoffice::art_index(&v, &page)
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
    query: Option<Query<ArtDetailQuery>>,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    let query = query.map(|Query(query)| query).unwrap_or_default();
    let notice = query.queued.map(|_| {
        "Regeneration started in the background. Refresh this page in a bit to see the updated art."
    });

    render_art_detail(&ctx, &v, id, notice, None).await
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
    Form(form): Form<ArtUpdateForm>,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    let title = form.title.trim().to_string();
    let prompt = form.prompt.trim().to_string();
    let model = normalize_model(&form.model);

    if title.is_empty() || prompt.is_empty() {
        return render_art_detail(
            &ctx,
            &v,
            id,
            None,
            Some("Title and prompt cannot be empty."),
        )
        .await;
    }

    arts::Model::update_details(
        &ctx.db,
        id,
        &ArtUpdateParams {
            title,
            prompt,
            model,
        },
    )
    .await?;

    Ok(Redirect::to(&format!("/backoffice/arts/{id}")).into_response())
}

#[debug_handler]
pub async fn replace(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    load_item(&ctx, id).await?;

    let ctx = ctx.clone();
    tokio::spawn(async move {
        if let Err(err) = art_service::replace_art(&ctx, id).await {
            error!(art_id = id, error = %err, "background art regeneration failed");
        }
    });

    Ok(Redirect::to(&format!("/backoffice/arts/{id}?queued=1")).into_response())
}

#[debug_handler]
pub async fn delete(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    arts::Model::delete_by_id(&ctx.db, id).await?;
    Ok(Redirect::to("/backoffice/arts").into_response())
}

#[debug_handler]
pub async fn mix_index(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
    query: Option<Query<MixListQuery>>,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    let query = query.map(|Query(query)| query).unwrap_or_default();
    let page = mixes::Model::find_backoffice_page(&ctx.db, query.page.unwrap_or(1)).await?;
    views::backoffice::mix_index(&v, &page)
}

#[debug_handler]
pub async fn mix_show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    let item = mixes::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    let art_ids = mixarts::Model::find_art_ids(&ctx.db, id).await?;

    views::backoffice::mix_detail(&v, &item, &art_ids)
}

#[debug_handler]
pub async fn mix_delete(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    jar: CookieJar,
) -> Result<Response> {
    if let Some(response) = require_auth(&ctx, &jar)? {
        return Ok(response);
    }

    mixes::Model::delete_by_id(&ctx.db, id).await?;
    Ok(Redirect::to("/backoffice/mixes").into_response())
}

fn normalize_model(model: &str) -> Option<String> {
    let model = model.trim();
    (!model.is_empty()).then(|| model.to_string())
}

fn require_auth(ctx: &AppContext, jar: &CookieJar) -> Result<Option<Response>> {
    if backoffice_auth::is_authenticated(ctx, jar)? {
        return Ok(None);
    }

    Ok(Some(backoffice_auth::redirect_to_login()))
}

async fn render_art_detail(
    ctx: &AppContext,
    v: &TeraView,
    id: i32,
    notice: Option<&str>,
    error: Option<&str>,
) -> Result<Response> {
    let item = load_item(ctx, id).await?;
    let previous_id = arts::Model::find_previous_id(&ctx.db, id).await?;
    let next_id = arts::Model::find_next_id(&ctx.db, id).await?;

    views::backoffice::art_detail(v, &item, previous_id, next_id, notice, error)
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<arts::Model> {
    arts::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)
}
