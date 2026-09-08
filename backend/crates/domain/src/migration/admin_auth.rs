//! P1: admin_users, admin_sessions, recovery_codes, audit_log.

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum AdminUsers {
    Table,
    Id,
    LoginCodePrefix,
    LoginCodeHash,
    PasswordHash,
    Email,
    TotpSecret,
    TotpEnabled,
    Role,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum AdminSessions {
    Table,
    Id,
    AdminUserId,
    RefreshHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum RecoveryCodes {
    Table,
    Id,
    AdminUserId,
    CodeHash,
    UsedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum AuditLog {
    Table,
    Id,
    ActorAdminId,
    Action,
    TargetType,
    TargetId,
    Meta,
    Ip,
    At,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        admin_users(m).await?;
        admin_sessions(m).await?;
        recovery_codes(m).await?;
        audit_log(m).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(AuditLog::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(RecoveryCodes::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(AdminSessions::Table).to_owned())
            .await?;
        m.drop_table(Table::drop().table(AdminUsers::Table).to_owned())
            .await?;
        m.get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS admin_role; DROP TYPE IF EXISTS admin_status;")
            .await?;
        Ok(())
    }
}

async fn admin_users(m: &SchemaManager<'_>) -> Result<(), DbErr> {
    m.get_connection()
        .execute_unprepared(
            "CREATE TYPE admin_role AS ENUM ('mother_admin', 'admin'); \
             CREATE TYPE admin_status AS ENUM ('active', 'locked');",
        )
        .await?;
    m.create_table(
        Table::create()
            .table(AdminUsers::Table)
            .col(
                uuid(AdminUsers::Id)
                    .primary_key()
                    .default(Expr::cust("gen_random_uuid()")),
            )
            .col(string_len(AdminUsers::LoginCodePrefix, 4).not_null())
            .col(text(AdminUsers::LoginCodeHash).not_null())
            .col(text(AdminUsers::PasswordHash).not_null())
            .col(text(AdminUsers::Email).null())
            .col(text(AdminUsers::TotpSecret).null())
            .col(boolean(AdminUsers::TotpEnabled).not_null().default(false))
            .col(
                enumeration(
                    AdminUsers::Role,
                    Alias::new("admin_role"),
                    [Alias::new("mother_admin"), Alias::new("admin")],
                )
                .not_null(),
            )
            .col(
                enumeration(
                    AdminUsers::Status,
                    Alias::new("admin_status"),
                    [Alias::new("active"), Alias::new("locked")],
                )
                .not_null()
                .default(Expr::cust("'active'")),
            )
            .col(
                timestamp_with_time_zone(AdminUsers::CreatedAt)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .col(
                timestamp_with_time_zone(AdminUsers::UpdatedAt)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .to_owned(),
    )
    .await?;
    m.create_index(
        Index::create()
            .name("idx_admin_users_prefix")
            .table(AdminUsers::Table)
            .col(AdminUsers::LoginCodePrefix)
            .to_owned(),
    )
    .await
}

async fn admin_sessions(m: &SchemaManager<'_>) -> Result<(), DbErr> {
    m.create_table(
        Table::create()
            .table(AdminSessions::Table)
            .col(
                uuid(AdminSessions::Id)
                    .primary_key()
                    .default(Expr::cust("gen_random_uuid()")),
            )
            .col(uuid(AdminSessions::AdminUserId).not_null())
            .col(text(AdminSessions::RefreshHash).not_null().unique_key())
            .col(timestamp_with_time_zone(AdminSessions::ExpiresAt).not_null())
            .col(timestamp_with_time_zone(AdminSessions::RevokedAt).null())
            .col(
                timestamp_with_time_zone(AdminSessions::CreatedAt)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .col(
                timestamp_with_time_zone(AdminSessions::UpdatedAt)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_sessions_admin")
                    .from(AdminSessions::Table, AdminSessions::AdminUserId)
                    .to(AdminUsers::Table, AdminUsers::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned(),
    )
    .await
}

async fn recovery_codes(m: &SchemaManager<'_>) -> Result<(), DbErr> {
    m.create_table(
        Table::create()
            .table(RecoveryCodes::Table)
            .col(
                uuid(RecoveryCodes::Id)
                    .primary_key()
                    .default(Expr::cust("gen_random_uuid()")),
            )
            .col(uuid(RecoveryCodes::AdminUserId).not_null())
            .col(text(RecoveryCodes::CodeHash).not_null())
            .col(timestamp_with_time_zone(RecoveryCodes::UsedAt).null())
            .col(
                timestamp_with_time_zone(RecoveryCodes::CreatedAt)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .col(
                timestamp_with_time_zone(RecoveryCodes::UpdatedAt)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_recovery_admin")
                    .from(RecoveryCodes::Table, RecoveryCodes::AdminUserId)
                    .to(AdminUsers::Table, AdminUsers::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned(),
    )
    .await
}

async fn audit_log(m: &SchemaManager<'_>) -> Result<(), DbErr> {
    m.create_table(
        Table::create()
            .table(AuditLog::Table)
            .col(
                uuid(AuditLog::Id)
                    .primary_key()
                    .default(Expr::cust("gen_random_uuid()")),
            )
            .col(uuid(AuditLog::ActorAdminId).null())
            .col(string(AuditLog::Action).not_null())
            .col(string(AuditLog::TargetType).null())
            .col(string(AuditLog::TargetId).null())
            .col(json_binary(AuditLog::Meta).null())
            .col(string(AuditLog::Ip).null())
            .col(
                timestamp_with_time_zone(AuditLog::At)
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_audit_actor")
                    .from(AuditLog::Table, AuditLog::ActorAdminId)
                    .to(AdminUsers::Table, AdminUsers::Id)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned(),
    )
    .await?;
    m.create_index(
        Index::create()
            .name("idx_audit_at")
            .table(AuditLog::Table)
            .col(AuditLog::At)
            .to_owned(),
    )
    .await?;
    m.create_index(
        Index::create()
            .name("idx_audit_actor")
            .table(AuditLog::Table)
            .col(AuditLog::ActorAdminId)
            .to_owned(),
    )
    .await
}
