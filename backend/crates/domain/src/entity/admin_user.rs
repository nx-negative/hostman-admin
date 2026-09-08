//! `admin_users` (§6). Login code stored as argon2 hash, AES-256-GCM encrypted (PII).

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "admin_role")]
pub enum Role {
    #[sea_orm(string_value = "mother_admin")]
    MotherAdmin,
    #[sea_orm(string_value = "admin")]
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "admin_status")]
pub enum Status {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "locked")]
    Locked,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "admin_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// First 4 chars of the normalized login code (plaintext lookup key).
    #[sea_orm(
        column_type = "String(StringLen::N(4))",
        index = "idx_admin_users_prefix"
    )]
    pub login_code_prefix: String,
    /// AES-256-GCM encrypted argon2id hash of the full login code (PII).
    pub login_code_hash: String,
    pub password_hash: String,
    /// AES-256-GCM encrypted (PII, optional).
    pub email: Option<String>,
    /// AES-256-GCM encrypted base32 TOTP secret (PII).
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub role: Role,
    pub status: Status,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::admin_session::Entity")]
    AdminSession,
    #[sea_orm(has_many = "super::recovery_code::Entity")]
    RecoveryCode,
}

impl Related<super::admin_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminSession.def()
    }
}

impl Related<super::recovery_code::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecoveryCode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
