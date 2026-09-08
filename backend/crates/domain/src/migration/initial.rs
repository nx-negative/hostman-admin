//! P0 baseline: `settings` table.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Settings {
    Table,
    Id,
    Key,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Settings::Table)
                .col(
                    uuid(Settings::Id)
                        .primary_key()
                        .default(Expr::cust("gen_random_uuid()")),
                )
                .col(string(Settings::Key).not_null().unique_key())
                .col(json_binary(Settings::Value).not_null())
                .col(
                    timestamp_with_time_zone(Settings::CreatedAt)
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    timestamp_with_time_zone(Settings::UpdatedAt)
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}
