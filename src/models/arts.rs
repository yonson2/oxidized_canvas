use base64::{engine::general_purpose, Engine as _};
use loco_rs::model::ModelResult;
use loco_rs::prelude::model;
use loco_rs::prelude::ActiveValue;
use loco_rs::prelude::ModelError;
use sea_orm::FromQueryResult;
use sea_orm::TransactionTrait;
use sea_orm::{entity::prelude::*, Order, QueryOrder, QuerySelect};

pub use super::_entities::arts::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::arts::ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            let mut this = self;
            this.uuid = ActiveValue::Set(Uuid::new_v4());
            Ok(this)
        } else {
            Ok(self)
        }
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
        tracing::info!(id = id, "Id that reached the fn");
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
}

pub struct ArtParams {
    pub image: String,
    pub prompt: String,
    pub title: String,
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
