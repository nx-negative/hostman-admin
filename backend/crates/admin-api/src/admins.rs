//! Admin management (§7.5) — mother_admin only.

use crate::audit;
use crate::state::{AppJson, AppState, AuthAdmin, require_mother};
use axum::{
    extract::{ConnectInfo, Path, State},
    response::{IntoResponse, Response},
};
use common::crypto::{
    encrypt_field, gen_login_code, gen_recovery_code, hash_secret, normalize_code, valid_code,
};
use common::error::{ApiError, ok};
use domain::entity::admin_user::{self, Role};
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReq {
    pub role: String,
    pub email: Option<String>,
}

pub async fn list(State(st): State<AppState>, admin: AuthAdmin) -> Response {
    if let Err(e) = require_mother(&admin) {
        return e.into_response();
    }
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let enc = match st.enc() {
        Ok(e) => e,
        Err(e) => return e.into_response(),
    };
    let rows = match admin_user::Entity::find().all(db).await {
        Ok(r) => r,
        Err(e) => return ApiError::internal(e.to_string()).into_response(),
    };
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|a| {
            json!({
                "id": a.id,
                "code_prefix": a.login_code_prefix,
                "email": a.email.as_deref().and_then(|e| common::crypto::decrypt_field(enc, e).ok()),
                "role": crate::state::role_str(a.role.clone()),
                "status": match a.status { admin_user::Status::Active => "active", admin_user::Status::Locked => "locked" },
                "totp_enabled": a.totp_enabled,
                "created_at": a.created_at.to_string(),
            })
        })
        .collect();
    ok(json!({"admins": out}))
}

pub async fn create(
    State(st): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    admin: AuthAdmin,
    AppJson(req): AppJson<CreateReq>,
) -> Response {
    if let Err(e) = require_mother(&admin) {
        return e.into_response();
    }
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let enc = match st.enc() {
        Ok(e) => e,
        Err(e) => return e.into_response(),
    };
    let role = match req.role.as_str() {
        "mother_admin" => Role::MotherAdmin,
        "admin" => Role::Admin,
        _ => return ApiError::validation("role must be mother_admin or admin").into_response(),
    };
    if let Some(email) = &req.email
        && (!email.contains('@') || email.len() > 254)
    {
        return ApiError::validation("invalid email").into_response();
    }
    let code = gen_login_code();
    let password = gen_recovery_code(); // 10 Crockford chars + dash — one-time password
    let prefix: String = normalize_code(&code).chars().take(4).collect();
    let now = time::OffsetDateTime::now_utc();
    let row = admin_user::ActiveModel {
        id: Set(Uuid::now_v7()),
        login_code_prefix: Set(prefix),
        login_code_hash: Set(encrypt_field(
            enc,
            &hash_secret(&normalize_code(&code)).unwrap_or_default(),
        )
        .unwrap_or_default()),
        password_hash: Set(hash_secret(&password).unwrap_or_default()),
        email: Set(req
            .email
            .as_deref()
            .map(|e| encrypt_field(enc, e).unwrap_or_default())),
        totp_secret: Set(None),
        totp_enabled: Set(false),
        role: Set(role),
        status: Set(admin_user::Status::Active),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = match admin_user::Entity::insert(row).exec(db).await {
        Ok(r) => r,
        Err(e) => return ApiError::internal(e.to_string()).into_response(),
    };
    let ip = addr.ip().to_string();
    audit::write(
        db,
        Some(admin.admin.id),
        "admin.created",
        Some("admin_user"),
        Some(created.last_insert_id.to_string().as_str()),
        json!({"role": req.role}),
        Some(ip),
    )
    .await;
    ok(json!({"id": created.last_insert_id, "login_code": code, "password": password}))
}

pub async fn delete(
    State(st): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    admin: AuthAdmin,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = require_mother(&admin) {
        return e.into_response();
    }
    if id == admin.admin.id {
        return ApiError::conflict("cannot remove self").into_response();
    }
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let target = match admin_user::Entity::find_by_id(id).one(db).await {
        Ok(Some(a)) => a,
        Ok(None) => return ApiError::not_found("admin not found").into_response(),
        Err(e) => return ApiError::internal(e.to_string()).into_response(),
    };
    if target.role == Role::MotherAdmin {
        return ApiError::forbidden("cannot remove a mother_admin").into_response();
    }
    if let Err(e) = admin_user::Entity::delete_by_id(id).exec(db).await {
        return ApiError::internal(e.to_string()).into_response();
    }
    let ip = addr.ip().to_string();
    let role = crate::state::role_str(target.role.clone()).to_string();
    audit::write(
        db,
        Some(admin.admin.id),
        "admin.deleted",
        Some("admin_user"),
        Some(id.to_string().as_str()),
        json!({"role": role}),
        Some(ip),
    )
    .await;
    ok(json!({"deleted": id.to_string()}))
}

/// Unused helper silencer — keeps `valid_code` import for future validation paths.
#[allow(dead_code)]
fn _code_ok(c: &str) -> bool {
    valid_code(&normalize_code(c))
}
