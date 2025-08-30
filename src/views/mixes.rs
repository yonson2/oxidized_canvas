use loco_rs::prelude::*;

use crate::models::{_entities::mixes, arts::ArtTitleId};

/// Render a single arts view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &mixes::Model, art_ids: &[i32]) -> Result<Response> {
    format::render().view(
        v,
        "mixes/show.html",
        serde_json::json!({"item": item, "art_ids": art_ids}),
    )
}

/// Render a the mix selection view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show_form(v: &impl ViewRenderer, title_ids: &[ArtTitleId]) -> Result<Response> {
    format::render().view(
        v,
        "mixes/form.html",
        serde_json::json!({"title_ids": title_ids}),
    )
}
