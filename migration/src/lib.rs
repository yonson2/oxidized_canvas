#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;

mod m20241007_173307_arts;
mod m20250828_101518_add_model_to_arts;
mod m20250830_091407_mixes;
mod m20250830_092716_mixarts;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20241007_173307_arts::Migration),
            Box::new(m20250828_101518_add_model_to_arts::Migration),
            Box::new(m20250830_091407_mixes::Migration),
            Box::new(m20250830_092716_mixarts::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}

