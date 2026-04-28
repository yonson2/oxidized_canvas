use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use loco_rs::prelude::*;
use sha2::{Digest, Sha256};

use crate::common::settings::Settings;

const BACKOFFICE_COOKIE: &str = "backoffice_session";

fn settings(ctx: &AppContext) -> Result<Settings> {
    Settings::from_json(
        &ctx.config
            .settings
            .clone()
            .ok_or(Error::Message("Invalid settings".into()))?,
    )
}

fn session_token(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update("oxidized_canvas_backoffice");
    hasher.update(password.trim());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

pub fn password_matches(ctx: &AppContext, candidate: &str) -> Result<bool> {
    let settings = settings(ctx)?;
    Ok(!settings.backoffice_password.trim().is_empty()
        && candidate.trim() == settings.backoffice_password.trim())
}

pub fn is_authenticated(ctx: &AppContext, jar: &CookieJar) -> Result<bool> {
    let settings = settings(ctx)?;
    let Some(cookie) = jar.get(BACKOFFICE_COOKIE) else {
        return Ok(false);
    };

    Ok(cookie.value() == session_token(&settings.backoffice_password))
}

pub fn session_cookie(ctx: &AppContext) -> Result<Cookie<'static>> {
    let settings = settings(ctx)?;
    Ok(Cookie::build((
        BACKOFFICE_COOKIE,
        session_token(&settings.backoffice_password),
    ))
    .path("/backoffice")
    .http_only(true)
    .same_site(SameSite::Lax)
    .build())
}

pub fn log_in(ctx: &AppContext, jar: CookieJar) -> Result<CookieJar> {
    Ok(jar.add(session_cookie(ctx)?))
}

pub fn log_out(jar: CookieJar) -> CookieJar {
    let mut cookie = Cookie::from(BACKOFFICE_COOKIE);
    cookie.set_path("/backoffice");
    jar.remove(cookie)
}

pub fn redirect_to_login() -> Response {
    Redirect::to("/backoffice/login").into_response()
}
