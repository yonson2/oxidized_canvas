use super::_entities::mixarts::{self, ActiveModel, Entity};
use loco_rs::model::ModelResult;
use sea_orm::FromQueryResult;
use sea_orm::{entity::prelude::*, ActiveValue, QuerySelect, TransactionTrait};
pub type Mixarts = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)

    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

impl super::_entities::mixarts::Model {
    /// Asynchronously creates the mix-art association.
    /// database.
    ///
    /// # Errors
    ///
    /// When could not save the art into the DB
    pub async fn create(db: &DatabaseConnection, params: &MixArtParams) -> ModelResult<()> {
        let txn = db.begin().await?;

        for art_id in params.art_ids.clone().into_iter() {
            mixarts::ActiveModel {
                art_id: ActiveValue::set(art_id),
                mix_id: ActiveValue::set(params.mix_id),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        Ok(())
    }

    /// Asynchronously finds all art ids related with a mix
    ///
    /// # Errors
    ///
    /// When could not save the art into the DB
    pub async fn find_art_ids(db: &DatabaseConnection, mix_id: i32) -> ModelResult<Vec<i32>> {
        let ids = mixarts::Entity::find()
            .filter(mixarts::Column::MixId.eq(mix_id))
            .select_only()
            .column(mixarts::Column::ArtId)
            .into_partial_model::<ArtId>()
            .all(db)
            .await?
            .iter()
            .map(|a| a.art_id)
            .collect();

        Ok(ids)
    }
}

pub struct MixArtParams {
    pub mix_id: i32,
    pub art_ids: Vec<i32>,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
struct ArtId {
    pub art_id: i32,
}
