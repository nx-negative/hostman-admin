//! Auth flow (§7.1): login → TOTP enroll/verify → JWT session; refresh; logout; me.

use crate::audit;
use crate::state::{
    ACCESS_COOKIE, ACCESS_TTL, AppJson, AppState, REFRESH_COOKIE, REFRESH_TTL, TEMP_TTL,
    cookie_value, decode_jwt, issue_jwt, role_str,
};
use axum::{
    extract::{ConnectInfo, State},
    response::{IntoResponse, Response},
};
use common::crypto::{
    decrypt_field, encrypt_field, gen_recovery_code, gen_token, gen_totp_secret, hash_secret,
    normalize_code, sha256_hex, totp_url, valid_code, verify_secret, verify_totp,
};
use common::error::{ApiError, Code, ok};
use domain::entity::{admin_session, admin_user, recovery_code};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::LazyLock;
use uuid::Uuid;

/// Burned on lookups for unknown codes to equalize timing.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| hash_secret("hostman-dummy").unwrap());

pub async fn current_admin(
    db: &sea_orm::DatabaseConnection,
    id: Uuid,
) -> Result<admin_user::Model, ApiError> {
    admin_user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::auth("no session"))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub login_code: String,
    pub password: String,
}

pub async fn login(
    State(st): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AppJson(req): AppJson<LoginReq>,
) -> Response {
    let ip = addr.ip().to_string();
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let enc = match st.enc() {
        Ok(e) => e,
        Err(e) => return e.into_response(),
    };
    if !st.login_limiter.check(&ip) {
        return ApiError::new(Code::RateLimit, "too many login attempts").into_response();
    }
    let code = normalize_code(&req.login_code);
    if !valid_code(&code) {
        return ApiError::validation("login_code must be 32 Crockford characters").into_response();
    }
    let prefix: String = code.chars().take(4).collect();
    let found = admin_user::Entity::find()
        .filter(admin_user::Column::LoginCodePrefix.eq(&prefix))
        .one(db)
        .await;
    let admin = match found {
        Ok(a) => a,
        Err(e) => return ApiError::internal(e.to_string()).into_response(),
    };

    let valid = match &admin {
        Some(a) => {
            let stored_code_hash = decrypt_field(enc, &a.login_code_hash).unwrap_or_default();
            verify_secret(&code, &stored_code_hash)
                && verify_secret(&req.password, &a.password_hash)
        }
        None => {
            let _ = verify_secret(&code, &DUMMY_HASH);
            false
        }
    };

    if !valid {
        if let Some(a) = &admin {
            let locked_secs = st.lockouts.fail(&a.id.to_string());
            audit::write(
                db,
                Some(a.id),
                "auth.login_failed",
                None,
                None,
                json!({"ip": ip, "locked_secs": locked_secs}),
                Some(ip.clone()),
            )
            .await;
        }
        return ApiError::auth("invalid credentials").into_response();
    }
    let admin = admin.expect("checked valid above");
    if admin.status == admin_user::Status::Locked {
        return ApiError::new(Code::Locked, "account locked").into_response();
    }
    st.lockouts.reset(&admin.id.to_string());
    audit::write(
        db,
        Some(admin.id),
        "auth.login",
        None,
        None,
        json!({"ip": ip}),
        Some(ip.clone()),
    )
    .await;

    if admin.totp_enabled {
        match issue_jwt(
            &st,
            admin.id,
            role_str(admin.role.clone()),
            "temp",
            None,
            TEMP_TTL,
        ) {
            Ok(t) => ok(json!({"state": "totp_required", "temp_token": t})),
            Err(e) => e.into_response(),
        }
    } else {
        let secret = match &admin.totp_secret {
            Some(s) => decrypt_field(enc, s).unwrap_or_else(|_| gen_totp_secret()),
            None => gen_totp_secret(),
        };
        if admin.totp_secret.is_none() {
            let mut am: admin_user::ActiveModel = admin.clone().into();
            am.totp_secret = Set(Some(encrypt_field(enc, &secret).unwrap_or_default()));
            am.updated_at = Set(time::OffsetDateTime::now_utc());
            if let Err(e) = am.update(db).await {
                return ApiError::internal(e.to_string()).into_response();
            }
        }
        let url = match totp_url(&secret, &prefix) {
            Ok(u) => u,
            Err(e) => return ApiError::internal(e.to_string()).into_response(),
        };
        match issue_jwt(
            &st,
            admin.id,
            role_str(admin.role.clone()),
            "temp",
            None,
            TEMP_TTL,
        ) {
            Ok(t) => ok(json!({"state": "totp_enroll", "temp_token": t, "otpauth_url": url})),
            Err(e) => e.into_response(),
        }
    }
}

#[derive(Deserialize)]
pub struct TotpReq {
    pub temp_token: String,
    pub code: String,
}

pub async fn totp_verify(
    State(st): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AppJson(req): AppJson<TotpReq>,
) -> Response {
    let ip = addr.ip().to_string();
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let enc = match st.enc() {
        Ok(e) => e,
        Err(e) => return e.into_response(),
    };
    let claims = match decode_jwt(&st, &req.temp_token, "temp") {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let mut admin = match current_admin(db, claims.sub).await {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };

    let mut via_recovery = false;
    let totp_ok = admin
        .totp_secret
        .as_deref()
        .and_then(|s| decrypt_field(enc, s).ok())
        .map(|secret| verify_totp(&secret, &req.code))
        .unwrap_or(false);
    if !totp_ok {
        // recovery code fallback: one-time use
        let hash = sha256_hex(&normalize_code(&req.code).replace('-', ""));
        match recovery_code::Entity::find()
            .filter(recovery_code::Column::AdminUserId.eq(admin.id))
            .filter(recovery_code::Column::UsedAt.is_null())
            .filter(recovery_code::Column::CodeHash.eq(&hash))
            .one(db)
            .await
        {
            Ok(Some(row)) => {
                let mut am: recovery_code::ActiveModel = row.into();
                am.used_at = Set(Some(time::OffsetDateTime::now_utc()));
                if am.update(db).await.is_err() {
                    return ApiError::internal("recovery update failed").into_response();
                }
                via_recovery = true;
            }
            Ok(None) => {
                audit::write(
                    db,
                    Some(admin.id),
                    "auth.totp_failed",
                    None,
                    None,
                    json!({"ip": ip}),
                    Some(ip.clone()),
                )
                .await;
                return ApiError::new(Code::Totp, "invalid code").into_response();
            }
            Err(e) => return ApiError::internal(e.to_string()).into_response(),
        }
    }

    let mut recovery_codes: Option<Vec<String>> = None;
    if !admin.totp_enabled {
        let codes: Vec<String> = (0..10).map(|_| gen_recovery_code()).collect();
        for c in &codes {
            let row = recovery_code::ActiveModel {
                id: Set(Uuid::now_v7()),
                admin_user_id: Set(admin.id),
                code_hash: Set(sha256_hex(&normalize_code(c).replace('-', ""))),
                used_at: Set(None),
                created_at: Set(time::OffsetDateTime::now_utc()),
                updated_at: Set(time::OffsetDateTime::now_utc()),
            };
            if let Err(e) = recovery_code::Entity::insert(row).exec(db).await {
                return ApiError::internal(e.to_string()).into_response();
            }
        }
        let mut am: admin_user::ActiveModel = admin.clone().into();
        am.totp_enabled = Set(true);
        am.updated_at = Set(time::OffsetDateTime::now_utc());
        if let Err(e) = am.update(db).await {
            return ApiError::internal(e.to_string()).into_response();
        }
        recovery_codes = Some(codes);
        admin.totp_enabled = true;
        audit::write(
            db,
            Some(admin.id),
            "auth.totp_enabled",
            None,
            None,
            json!({}),
            Some(ip.clone()),
        )
        .await;
    }
    if via_recovery {
        audit::write(
            db,
            Some(admin.id),
            "auth.recovery_used",
            None,
            None,
            json!({}),
            Some(ip.clone()),
        )
        .await;
    }
    audit::write(
        db,
        Some(admin.id),
        "auth.session_issued",
        None,
        None,
        json!({}),
        Some(ip.clone()),
    )
    .await;

    match issue_session(&st, db, &admin, true).await {
        Ok((access, refresh)) => session_response(&admin, access, refresh, recovery_codes),
        Err(e) => e.into_response(),
    }
}

#[derive(Serialize)]
pub struct AdminOut {
    pub id: Uuid,
    pub role: String,
    pub code_prefix: String,
    pub totp_enabled: bool,
}

pub fn admin_out(a: &admin_user::Model) -> AdminOut {
    AdminOut {
        id: a.id,
        role: role_str(a.role.clone()).to_string(),
        code_prefix: a.login_code_prefix.clone(),
        totp_enabled: a.totp_enabled,
    }
}

/// Create a session row + access/refresh pair. `revoke_others` (true on fresh login) kills all
/// other sessions for the admin → single-active-session ("one device at a time", §7.4).
async fn issue_session(
    st: &AppState,
    db: &sea_orm::DatabaseConnection,
    admin: &admin_user::Model,
    revoke_others: bool,
) -> Result<(String, String), ApiError> {
    let sid = Uuid::now_v7();
    let refresh = gen_token();
    let now = time::OffsetDateTime::now_utc();
    let row = admin_session::ActiveModel {
        id: Set(sid),
        admin_user_id: Set(admin.id),
        refresh_hash: Set(sha256_hex(&refresh)),
        expires_at: Set(now + REFRESH_TTL),
        revoked_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    admin_session::Entity::insert(row)
        .exec(db)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if revoke_others {
        admin_session::Entity::update_many()
            .set(admin_session::ActiveModel {
                revoked_at: Set(Some(now)),
                ..Default::default()
            })
            .filter(admin_session::Column::AdminUserId.eq(admin.id))
            .filter(admin_session::Column::RevokedAt.is_null())
            .filter(admin_session::Column::Id.ne(sid))
            .exec(db)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }
    let access = issue_jwt(
        st,
        admin.id,
        role_str(admin.role.clone()),
        "access",
        Some(sid),
        ACCESS_TTL,
    )?;
    Ok((access, refresh))
}

/// Session cookies carry NO Max-Age/Expires → browser-session-scoped (§7.4).
fn cookie_pair(access: &str, refresh: &str) -> Vec<String> {
    vec![
        format!("{ACCESS_COOKIE}={access}; Path=/; HttpOnly; SameSite=Lax"),
        format!("{REFRESH_COOKIE}={refresh}; Path=/api/v1; HttpOnly; SameSite=Lax"),
    ]
}

fn session_response(
    admin: &admin_user::Model,
    access: String,
    refresh: String,
    recovery_codes: Option<Vec<String>>,
) -> Response {
    use axum::http::header::SET_COOKIE;
    let mut res = ok(json!({
        "admin": admin_out(admin),
        "recovery_codes": recovery_codes,
    }));
    for c in cookie_pair(&access, &refresh) {
        if let Ok(v) = axum::http::HeaderValue::from_str(&c) {
            res.headers_mut().append(SET_COOKIE, v);
        }
    }
    res
}

pub async fn refresh(State(st): State<AppState>, jar: axum_extra::extract::CookieJar) -> Response {
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let token = match jar.get(REFRESH_COOKIE) {
        Some(c) => c.value().to_string(),
        None => return ApiError::auth("no refresh token").into_response(),
    };
    let row = match admin_session::Entity::find()
        .filter(admin_session::Column::RefreshHash.eq(sha256_hex(&token)))
        .filter(admin_session::Column::RevokedAt.is_null())
        .filter(admin_session::Column::ExpiresAt.gt(time::OffsetDateTime::now_utc()))
        .one(db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return ApiError::auth("invalid refresh token").into_response(),
        Err(e) => return ApiError::internal(e.to_string()).into_response(),
    };
    let mut revoke: admin_session::ActiveModel = row.clone().into();
    revoke.revoked_at = Set(Some(time::OffsetDateTime::now_utc()));
    if let Err(e) = revoke.update(db).await {
        return ApiError::internal(e.to_string()).into_response();
    }
    let admin = match current_admin(db, row.admin_user_id).await {
        Ok(a) => a,
        Err(e) => return e.into_response(),
    };
    if admin.status == admin_user::Status::Locked {
        return ApiError::new(Code::Locked, "account locked").into_response();
    }
    match issue_session(&st, db, &admin, false).await {
        Ok((access, new_refresh)) => session_response(&admin, access, new_refresh, None),
        Err(e) => e.into_response(),
    }
}

pub async fn logout(State(st): State<AppState>, jar: axum_extra::extract::CookieJar) -> Response {
    if let Some(db) = st.db.as_ref()
        && let Some(c) = jar.get(REFRESH_COOKIE)
        && let Ok(Some(row)) = admin_session::Entity::find()
            .filter(admin_session::Column::RefreshHash.eq(sha256_hex(c.value())))
            .filter(admin_session::Column::RevokedAt.is_null())
            .one(db)
            .await
    {
        let mut revoke: admin_session::ActiveModel = row.into();
        revoke.revoked_at = Set(Some(time::OffsetDateTime::now_utc()));
        let _ = revoke.update(db).await;
    }
    let mut res = ok(json!({"logged_out": true}));
    use axum::http::header::SET_COOKIE;
    for name in [ACCESS_COOKIE, REFRESH_COOKIE] {
        let c = format!("{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
        if let Ok(v) = axum::http::HeaderValue::from_str(&c) {
            res.headers_mut().append(SET_COOKIE, v);
        }
    }
    res
}

pub async fn me(State(st): State<AppState>, parts: axum::http::request::Parts) -> Response {
    let token = match cookie_value(&parts, ACCESS_COOKIE) {
        Some(t) => t,
        None => return ApiError::auth("no session").into_response(),
    };
    let claims = match decode_jwt(&st, &token, "access") {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let db = match st.db() {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    if let Some(sid) = claims.sid {
        #[allow(clippy::collapsible_if)]
        if let Err(e) = crate::state::session_valid(db, sid, claims.sub).await {
            return e.into_response();
        }
    }
    match current_admin(db, claims.sub).await {
        Ok(admin) => ok(json!({"admin": admin_out(&admin)})),
        Err(e) => e.into_response(),
    }
}
