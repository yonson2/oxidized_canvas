use loco_rs::prelude::*;

use crate::models::_entities::arts;

/// Render a list view of arts.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<arts::Model>) -> Result<Response> {
    format::render().view(v, "arts/list.html", serde_json::json!({"items": items}))
}

/// Render a single arts view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &arts::Model) -> Result<Response> {
    format::render().view(v, "arts/show.html", serde_json::json!({"item": item}))
}

/// Render a arts create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "arts/create.html", serde_json::json!({}))
}

/// Render a arts edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &arts::Model) -> Result<Response> {
    format::render().view(v, "arts/edit.html", serde_json::json!({"item": item}))
}
