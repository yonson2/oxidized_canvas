use loco_rs::prelude::*;

use crate::models::{
    arts::{self, BackofficeArtList, BackofficeStats},
    mixes::{self, BackofficeMixList},
};

pub fn login(v: &impl ViewRenderer, error: Option<&str>) -> Result<Response> {
    format::render().view(
        v,
        "backoffice/login.html",
        serde_json::json!({"error": error, "hide_nav": true}),
    )
}

pub fn dashboard(
    v: &impl ViewRenderer,
    stats: &BackofficeStats,
    recent_arts: &[arts::Model],
    recent_mixes: &[mixes::Model],
) -> Result<Response> {
    format::render().view(
        v,
        "backoffice/dashboard.html",
        serde_json::json!({
            "stats": stats,
            "recent_arts": recent_arts,
            "recent_mixes": recent_mixes,
        }),
    )
}

pub fn art_index(v: &impl ViewRenderer, page: &BackofficeArtList) -> Result<Response> {
    format::render().view(v, "backoffice/arts.html", serde_json::json!({"page": page}))
}

pub fn art_detail(
    v: &impl ViewRenderer,
    item: &arts::Model,
    previous_id: Option<i32>,
    next_id: Option<i32>,
    notice: Option<&str>,
    error: Option<&str>,
) -> Result<Response> {
    format::render().view(
        v,
        "backoffice/art.html",
        serde_json::json!({
            "item": item,
            "previous_id": previous_id,
            "next_id": next_id,
            "notice": notice,
            "error": error,
        }),
    )
}

pub fn mix_index(v: &impl ViewRenderer, page: &BackofficeMixList) -> Result<Response> {
    format::render().view(
        v,
        "backoffice/mixes.html",
        serde_json::json!({"page": page}),
    )
}

pub fn mix_detail(v: &impl ViewRenderer, item: &mixes::Model, art_ids: &[i32]) -> Result<Response> {
    format::render().view(
        v,
        "backoffice/mix.html",
        serde_json::json!({"item": item, "art_ids": art_ids}),
    )
}
