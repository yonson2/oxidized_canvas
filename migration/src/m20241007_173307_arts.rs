use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_tz(Arts::Table)
                    .col(pk_auto(Arts::Id))
                    .col(string(Arts::Image))
                    .col(string(Arts::Prompt))
                    .col(string(Arts::Title))
                    .col(uuid(Arts::Uuid))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Arts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Arts {
    Table,
    Id,
    Image,
    Prompt,
    Title,
    Uuid,
    
}


