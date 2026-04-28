use base64::{engine::general_purpose, Engine as _};
use loco_rs::model::query;
use loco_rs::model::query::PageResponse;
use loco_rs::model::query::PaginationQuery;
use loco_rs::model::ModelResult;
use loco_rs::prelude::model;
use loco_rs::prelude::ActiveModelTrait;
use loco_rs::prelude::ActiveValue;
use loco_rs::prelude::ModelError;
use loco_rs::Error;
use sea_orm::FromQueryResult;
use sea_orm::TransactionTrait;
use sea_orm::{
    entity::prelude::*, ColumnTrait, Condition, EntityTrait, Order, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::Deserialize;
use serde::Serialize;

use super::_entities::mixes;

pub use super::_entities::arts::{self, ActiveModel, Entity, Model};

pub const PAGE_SIZE: u64 = 5;
pub const BACKOFFICE_PAGE_SIZE: u64 = 24;

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::arts::ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut this = self;
        if insert {
            this.uuid = ActiveValue::Set(Uuid::new_v4());
        } else if this.updated_at.is_unchanged() {
            this.updated_at = ActiveValue::Set(chrono::Utc::now().into());
        }
        Ok(this)
    }
}

pub trait ModelVec {
    fn to_formatted_prompts(&self) -> String;
    fn to_formatted_titles(&self) -> String;
}

impl ModelVec for [Model] {
    fn to_formatted_prompts(&self) -> String {
        self.iter()
            .enumerate()
            .map(|(i, a)| format![" - Prompt {}: {}", i + 1, a.prompt.as_str(),])
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    fn to_formatted_titles(&self) -> String {
        self.iter()
            .enumerate()
            .map(|(i, a)| format![" - Title {}: {}", i + 1, a.title.as_str(),])
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

impl super::_entities::arts::Model {
    /// Asynchronously creates an art.
    /// database.
    ///
    /// # Errors
    ///
    /// When could not save the art into the DB
    pub async fn create(db: &DatabaseConnection, params: &ArtParams) -> ModelResult<Self> {
        let txn = db.begin().await?;

        let art = arts::ActiveModel {
            image: ActiveValue::set(params.image.to_string()),
            prompt: ActiveValue::set(params.prompt.to_string()),
            title: ActiveValue::set(params.title.to_string()),
            model: ActiveValue::set(params.model.clone()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(art)
    }

    ///
    /// fetches the most recently created `arts::Model`
    ///
    /// # Errors
    ///
    /// When could not find art or DB query error
    pub async fn find_latest(db: &DatabaseConnection) -> ModelResult<Self> {
        let arts = arts::Entity::find()
            .order_by_desc(arts::Column::CreatedAt)
            .limit(1)
            .one(db)
            .await?;

        arts.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// finds just the latest id (to see if we should display the "next" button)
    ///
    /// # Errors
    ///
    /// When could not find latest art or DB query error
    pub async fn find_latest_id(db: &DatabaseConnection) -> ModelResult<i32> {
        let ArtId { id } = arts::Entity::find()
            .order_by_desc(arts::Column::CreatedAt)
            .limit(1)
            .select_only()
            .column(arts::Column::Id)
            .into_partial_model::<ArtId>()
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(id)
    }

    /// finds all of the arts with the specified ids
    ///
    /// # Errors
    ///
    /// On DB query error
    pub async fn find_in(db: &DatabaseConnection, ids: Vec<i32>) -> ModelResult<Vec<Model>> {
        let arts = arts::Entity::find()
            .filter(arts::Column::Id.is_in(ids))
            .all(db)
            .await?;

        Ok(arts)
    }

    /// finds the ids of all of the created arts
    ///
    /// # Errors
    ///
    /// When could not find arts or DB query error
    pub async fn find_ids(db: &DatabaseConnection) -> ModelResult<Vec<i32>> {
        let ids = arts::Entity::find()
            .order_by_asc(arts::Column::CreatedAt)
            .select_only()
            .column(arts::Column::Id)
            .into_partial_model::<ArtId>()
            .all(db)
            .await?
            .iter()
            .map(|a| a.id)
            .collect();

        Ok(ids)
    }

    /// finds n arts at random
    ///
    /// # Errors
    ///
    /// When could not find art or DB query error
    pub async fn find_n_random(db: &DatabaseConnection, n: u64) -> ModelResult<Vec<Self>> {
        let arts = arts::Entity::find()
            .order_by(Expr::cust("RANDOM()"), Order::Asc)
            .limit(n)
            .all(db)
            .await?;

        Ok(arts)
    }

    /// finds latest n arts ordered by creation date (most recent first)
    /// # Errors
    ///
    /// When could not find art or DB query error
    pub async fn find_n_latest(db: &DatabaseConnection, n: u64) -> ModelResult<Vec<Self>> {
        let arts = arts::Entity::find()
            .order_by_desc(arts::Column::CreatedAt)
            .limit(n)
            .all(db)
            .await?;

        Ok(arts)
    }

    /// finds an art an returns just its base64 encoded image
    /// # Errors
    ///
    /// When db fails or when the item is missing
    pub async fn find_img_slice_by_id(db: &DatabaseConnection, id: u32) -> ModelResult<Vec<u8>> {
        let image = match arts::Entity::find()
            .filter(model::query::condition().eq(arts::Column::Id, id).build())
            .limit(1)
            .select_only()
            .column(arts::Column::Image)
            .into_partial_model::<ArtImage>()
            .one(db)
            .await
        {
            Ok(Some(ArtImage { image })) => Ok(image),
            Ok(None) => Err(ModelError::EntityNotFound),
            Err(e) => {
                tracing::error!(error = e.to_string(), "Error querying db");
                return Err(ModelError::DbErr(e));
            }
        }?;

        let img = match general_purpose::STANDARD.decode(image) {
            Ok(bytes) => bytes,
            Err(e) => return Err(ModelError::Any(Box::new(e))),
        };

        Ok(img)
    }

    /// finds the ids and titles of all of the created arts
    ///
    /// # Errors
    ///
    /// When could not find arts or DB query error
    pub async fn find_all_title_ids(db: &DatabaseConnection) -> ModelResult<Vec<ArtTitleId>> {
        let title_ids = arts::Entity::find()
            .select_only()
            .columns([arts::Column::Id, arts::Column::Title])
            .order_by_desc(arts::Column::CreatedAt)
            .into_partial_model::<ArtTitleId>()
            .all(db)
            .await?;

        Ok(title_ids)
    }

    /// fetches the most recently created `arts::Model`s
    /// the returned data is paginated.
    ///
    /// # Errors
    ///
    /// When could not find arts or DB query error
    pub async fn find_all_latest(
        db: &DatabaseConnection,
        pagination: &PaginationQuery,
    ) -> Result<PageResponse<ArtTitleId>, Error> {
        query::fetch_page(
            db,
            arts::Entity::find()
                .select_only()
                .columns([arts::Column::Id, arts::Column::Title])
                .order_by_desc(arts::Column::Id)
                .into_partial_model::<ArtTitleId>(),
            pagination,
        )
        .await
    }

    /// fetches `arts::Model`s before the given id.
    /// the returned data is paginated.
    ///
    /// # Errors
    ///
    /// When could not find arts or DB query error
    pub async fn find_before_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Vec<ArtTitleId>, Error> {
        let arts = arts::Entity::find()
            .select_only()
            .columns([arts::Column::Id, arts::Column::Title])
            .cursor_by(arts::Column::Id)
            .into_partial_model::<ArtTitleId>()
            .before(id)
            .last(PAGE_SIZE)
            .all(db)
            .await?;

        Ok(arts.into_iter().rev().collect())
    }

    /// fetches `arts::Model`s before the given id.
    /// the returned data is paginated.
    ///
    /// # Errors
    ///
    /// When could not find arts or DB query error
    pub async fn find_after_id(db: &DatabaseConnection, id: i32) -> Result<Vec<ArtTitleId>, Error> {
        let arts = arts::Entity::find()
            .select_only()
            .columns([arts::Column::Id, arts::Column::Title])
            .cursor_by(arts::Column::Id)
            .into_partial_model::<ArtTitleId>()
            .after(id)
            .first(PAGE_SIZE)
            .all(db)
            .await?;

        Ok(arts)
    }

    pub async fn find_backoffice_page(
        db: &DatabaseConnection,
        page: u64,
        search: Option<&str>,
    ) -> Result<BackofficeArtList, Error> {
        let page = page.max(1);
        let search = search
            .map(str::trim)
            .filter(|term| !term.is_empty())
            .map(ToOwned::to_owned);

        let mut query = arts::Entity::find().order_by_desc(arts::Column::CreatedAt);
        if let Some(term) = &search {
            query = query.filter(backoffice_search_condition(term));
        }

        let paginator = query.paginate(db, BACKOFFICE_PAGE_SIZE);
        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        let current_page = match total_pages {
            0 => 1,
            _ => page.min(total_pages),
        };
        let items = paginator.fetch_page(current_page.saturating_sub(1)).await?;

        Ok(BackofficeArtList {
            items,
            page: current_page,
            total_pages,
            total_items,
            query: search,
            has_previous: current_page > 1,
            has_next: total_pages > 0 && current_page < total_pages,
            previous_page: (current_page > 1).then_some(current_page - 1),
            next_page: (total_pages > 0 && current_page < total_pages).then_some(current_page + 1),
        })
    }

    pub async fn backoffice_stats(db: &DatabaseConnection) -> Result<BackofficeStats, Error> {
        let now = chrono::Utc::now();
        let total_arts = arts::Entity::find().count(db).await?;
        let total_mixes = mixes::Entity::find().count(db).await?;
        let arts_last_7_days = arts::Entity::find()
            .filter(arts::Column::CreatedAt.gte(now - chrono::Duration::days(7)))
            .count(db)
            .await?;
        let arts_last_30_days = arts::Entity::find()
            .filter(arts::Column::CreatedAt.gte(now - chrono::Duration::days(30)))
            .count(db)
            .await?;
        let updated_last_30_days = arts::Entity::find()
            .filter(arts::Column::UpdatedAt.gte(now - chrono::Duration::days(30)))
            .count(db)
            .await?;
        let newest_art_at = arts::Entity::find()
            .order_by_desc(arts::Column::CreatedAt)
            .limit(1)
            .select_only()
            .column(arts::Column::CreatedAt)
            .into_partial_model::<ArtCreatedAt>()
            .one(db)
            .await?
            .map(|art| art.created_at);
        let oldest_art_at = arts::Entity::find()
            .order_by_asc(arts::Column::CreatedAt)
            .limit(1)
            .select_only()
            .column(arts::Column::CreatedAt)
            .into_partial_model::<ArtCreatedAt>()
            .one(db)
            .await?
            .map(|art| art.created_at);
        let newest_update_at = arts::Entity::find()
            .order_by_desc(arts::Column::UpdatedAt)
            .limit(1)
            .select_only()
            .column(arts::Column::UpdatedAt)
            .into_partial_model::<ArtUpdatedAt>()
            .one(db)
            .await?
            .map(|art| art.updated_at);
        let models = arts::Entity::find()
            .select_only()
            .column(arts::Column::Model)
            .column_as(Expr::col(arts::Column::Id).count(), "count")
            .group_by(arts::Column::Model)
            .order_by_desc(Expr::col(arts::Column::Id).count())
            .into_model::<BackofficeModelStat>()
            .all(db)
            .await?;

        Ok(BackofficeStats {
            total_arts,
            total_mixes,
            arts_last_7_days,
            arts_last_30_days,
            updated_last_30_days,
            newest_art_at,
            oldest_art_at,
            newest_update_at,
            models,
        })
    }

    pub async fn update_details(
        db: &DatabaseConnection,
        id: i32,
        params: &ArtUpdateParams,
    ) -> ModelResult<Self> {
        let art = arts::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)?;

        let mut art_active_model: ActiveModel = art.into();
        art_active_model.title = ActiveValue::set(params.title.clone());
        art_active_model.prompt = ActiveValue::set(params.prompt.clone());
        art_active_model.model = ActiveValue::set(params.model.clone());
        art_active_model.updated_at = ActiveValue::set(chrono::Utc::now().into());

        art_active_model.update(db).await.map_err(Into::into)
    }

    pub async fn delete_by_id(db: &DatabaseConnection, id: i32) -> ModelResult<()> {
        let art = arts::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)?;

        art.delete(db).await?;
        Ok(())
    }

    pub async fn find_previous_id(db: &DatabaseConnection, id: i32) -> ModelResult<Option<i32>> {
        Ok(arts::Entity::find()
            .filter(arts::Column::Id.lt(id))
            .order_by_desc(arts::Column::Id)
            .select_only()
            .column(arts::Column::Id)
            .into_partial_model::<ArtId>()
            .one(db)
            .await?
            .map(|art| art.id))
    }

    pub async fn find_next_id(db: &DatabaseConnection, id: i32) -> ModelResult<Option<i32>> {
        Ok(arts::Entity::find()
            .filter(arts::Column::Id.gt(id))
            .order_by_asc(arts::Column::Id)
            .select_only()
            .column(arts::Column::Id)
            .into_partial_model::<ArtId>()
            .one(db)
            .await?
            .map(|art| art.id))
    }
}

pub struct ArtParams {
    pub image: String,
    pub prompt: String,
    pub title: String,
    pub model: Option<String>,
}

pub struct ArtUpdateParams {
    pub title: String,
    pub prompt: String,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackofficeArtList {
    pub items: Vec<Model>,
    pub page: u64,
    pub total_pages: u64,
    pub total_items: u64,
    pub query: Option<String>,
    pub has_previous: bool,
    pub has_next: bool,
    pub previous_page: Option<u64>,
    pub next_page: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackofficeStats {
    pub total_arts: u64,
    pub total_mixes: u64,
    pub arts_last_7_days: u64,
    pub arts_last_30_days: u64,
    pub updated_last_30_days: u64,
    pub newest_art_at: Option<DateTimeWithTimeZone>,
    pub oldest_art_at: Option<DateTimeWithTimeZone>,
    pub newest_update_at: Option<DateTimeWithTimeZone>,
    pub models: Vec<BackofficeModelStat>,
}

#[derive(FromQueryResult, Serialize, Deserialize, Debug)]
pub struct BackofficeModelStat {
    pub model: Option<String>,
    pub count: i64,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
struct ArtId {
    pub id: i32,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
struct ArtImage {
    pub image: String,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
struct ArtCreatedAt {
    pub created_at: DateTimeWithTimeZone,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
struct ArtUpdatedAt {
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(DerivePartialModel, FromQueryResult, Serialize, Deserialize, Debug)]
#[sea_orm(entity = "Entity")]
pub struct ArtTitleId {
    pub id: i32,
    pub title: String,
}

impl From<arts::Model> for ArtTitleId {
    fn from(value: arts::Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
        }
    }
}

fn backoffice_search_condition(term: &str) -> Condition {
    let mut condition = Condition::any()
        .add(arts::Column::Title.contains(term))
        .add(arts::Column::Prompt.contains(term))
        .add(arts::Column::Model.contains(term));

    if let Ok(id) = term.parse::<i32>() {
        condition = condition.add(arts::Column::Id.eq(id));
    }

    condition
}
