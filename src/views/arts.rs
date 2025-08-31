use loco_rs::{
    controller::views::pagination::{Pager, PagerMeta},
    model::query::{PageResponse, PaginationQuery},
    prelude::*,
};
use serde::{Deserialize, Serialize};

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

/// Renders a snap-scrolling, infinitely lazy-loaded view of the images.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show_infinite(v: &impl ViewRenderer, items: &[ArtTitleId]) -> Result<Response> {
    format::render().view(v, "arts/infinite.html", serde_json::json!({"items": items}))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListResponse {
    id: i32,
    title: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationResponse {}

impl From<ArtTitleId> for ListResponse {
    fn from(art: ArtTitleId) -> Self {
        Self {
            id: art.id,
            title: art.title.clone(),
            url: format!("/img/{}.webp", art.id),
        }
    }
}

impl PaginationResponse {
    #[must_use]
    pub fn response(
        data: PageResponse<ArtTitleId>,
        pagination_query: &PaginationQuery,
    ) -> Pager<Vec<ListResponse>> {
        Pager {
            results: data
                .page
                .into_iter()
                .map(ListResponse::from)
                .collect::<Vec<ListResponse>>(),
            info: PagerMeta {
                page: pagination_query.page,
                page_size: pagination_query.page_size,
                total_pages: data.total_pages,
            },
        }
    }
}
