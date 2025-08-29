use loco_rs::prelude::*;

use crate::models::{_entities::arts, arts::ArtTitleId};

/// Render a single arts view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &arts::Model, latest: bool) -> Result<Response> {
    format::render().view(
        v,
        "arts/show.html",
        serde_json::json!({"item": item, "latest": latest}),
    )
}

/// Render a the mix selection view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show_mix(v: &impl ViewRenderer, title_ids: &[ArtTitleId]) -> Result<Response> {
    format::render().view(v, "arts/show_mix.html", serde_json::json!({"title_ids": title_ids}))
}
