use super::_entities::users::{ActiveModel, Entity};
use sea_orm::entity::prelude::*;
pub type Users = Entity;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}
