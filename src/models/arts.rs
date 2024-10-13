use loco_rs::model::ModelResult;
use loco_rs::prelude::ActiveValue;
use loco_rs::prelude::ModelError;
use sea_orm::{entity::prelude::*, QueryOrder, QuerySelect};
use sea_orm::{Order, TransactionTrait};

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
}

pub struct ArtParams {
    pub image: String,
    pub prompt: String,
    pub title: String,
}
