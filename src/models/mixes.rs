pub use super::_entities::mixes::{self, ActiveModel, Entity, Model};
use base64::engine::general_purpose;
use base64::Engine;
use loco_rs::model::{self, ModelError, ModelResult};
use loco_rs::Error;
use sea_orm::FromQueryResult;
use sea_orm::{
    entity::prelude::*, ActiveValue, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect,
    TransactionTrait,
};
pub type Mixes = Entity;

pub const BACKOFFICE_PAGE_SIZE: u64 = 24;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)

    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
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

impl super::_entities::mixes::Model {
    /// Asynchronously creates a mix.
    /// database.
    ///
    /// # Errors
    ///
    /// When could not save the art into the DB
    pub async fn create(db: &DatabaseConnection, params: &MixParams) -> ModelResult<Self> {
        let txn = db.begin().await?;

        let art = mixes::ActiveModel {
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

    /// finds an art an returns just its base64 encoded image
    /// # Errors
    ///
    /// When db fails or when the item is missing
    pub async fn find_img_slice_by_id(db: &DatabaseConnection, id: u32) -> ModelResult<Vec<u8>> {
        let image = match mixes::Entity::find()
            .filter(model::query::condition().eq(mixes::Column::Id, id).build())
            .limit(1)
            .select_only()
            .column(mixes::Column::Image)
            .into_partial_model::<MixImage>()
            .one(db)
            .await
        {
            Ok(Some(MixImage { image })) => Ok(image),
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

    pub async fn find_n_latest(db: &DatabaseConnection, n: u64) -> ModelResult<Vec<Self>> {
        mixes::Entity::find()
            .order_by_desc(mixes::Column::CreatedAt)
            .limit(n)
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn find_backoffice_page(
        db: &DatabaseConnection,
        page: u64,
    ) -> Result<BackofficeMixList, Error> {
        let page = page.max(1);
        let paginator = mixes::Entity::find()
            .order_by_desc(mixes::Column::CreatedAt)
            .paginate(db, BACKOFFICE_PAGE_SIZE);
        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        let current_page = match total_pages {
            0 => 1,
            _ => page.min(total_pages),
        };
        let items = paginator.fetch_page(current_page.saturating_sub(1)).await?;

        Ok(BackofficeMixList {
            items,
            page: current_page,
            total_pages,
            total_items,
            previous_page: (current_page > 1).then_some(current_page - 1),
            next_page: (total_pages > 0 && current_page < total_pages).then_some(current_page + 1),
        })
    }

    pub async fn delete_by_id(db: &DatabaseConnection, id: i32) -> ModelResult<()> {
        let mix = mixes::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)?;

        mix.delete(db).await?;
        Ok(())
    }
}

pub struct MixParams {
    pub image: String,
    pub prompt: String,
    pub title: String,
    pub model: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BackofficeMixList {
    pub items: Vec<mixes::Model>,
    pub page: u64,
    pub total_pages: u64,
    pub total_items: u64,
    pub previous_page: Option<u64>,
    pub next_page: Option<u64>,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
struct MixImage {
    pub image: String,
}
