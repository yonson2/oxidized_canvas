use sea_orm::entity::prelude::*;
use super::_entities::users::{ActiveModel, Entity};
pub type Users = Entity;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}
