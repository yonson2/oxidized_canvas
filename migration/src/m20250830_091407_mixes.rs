use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_tz(Mixes::Table)
                    .col(pk_auto(Mixes::Id))
                    .col(string(Mixes::Image))
                    .col(string(Mixes::Prompt))
                    .col(string(Mixes::Title))
                    .col(uuid(Mixes::Uuid))
                    .col(string(Mixes::Model))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mixes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Mixes {
    Table,
    Id,
    Image,
    Prompt,
    Title,
    Uuid,
    Model,
}
