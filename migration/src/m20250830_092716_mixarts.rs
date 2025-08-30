use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_tz(Mixarts::Table)
                    .col(pk_auto(Mixarts::Id))
                    .col(integer(Mixarts::ArtId))
                    .col(integer(Mixarts::MixId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mixarts-art_ids")
                            .from(Mixarts::Table, Mixarts::ArtId)
                            .to(Arts::Table, Arts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mixarts-mix_ids")
                            .from(Mixarts::Table, Mixarts::MixId)
                            .to(Mixes::Table, Mixes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mixarts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Mixarts {
    Table,
    Id,
    ArtId,
    MixId,
}

#[derive(DeriveIden)]
enum Arts {
    Table,
    Id,
}
#[derive(DeriveIden)]
enum Mixes {
    Table,
    Id,
}
