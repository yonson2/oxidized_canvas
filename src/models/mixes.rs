use super::_entities::mixes::{self, ActiveModel, Entity};
use loco_rs::model::ModelResult;
use sea_orm::{entity::prelude::*, ActiveValue, TransactionTrait};
pub type Mixes = Entity;

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
}

pub struct MixParams {
    pub image: String,
    pub prompt: String,
    pub title: String,
    pub model: String,
}
