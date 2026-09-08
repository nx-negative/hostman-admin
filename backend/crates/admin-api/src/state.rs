//! Shared request state, JWT claims, auth extractor, cookie helpers.

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use common::crypto::{self, FieldKey};
use common::error::{ApiError, Code};
use common::ratelimit::{Lockout, RateLimiter};
use domain::entity::admin_user::{self, Status};
use ed25519_dalek::SigningKey;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub const ACCESS_COOKIE: &str = "hm_access";
pub const REFRESH_COOKIE: &str = "hm_refresh";
pub const ACCESS_TTL: Duration = Duration::from_secs(15 * 60);
pub const REFRESH_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
pub const TEMP_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Clone)]
pub struct AppState {
    pub db: Option<DatabaseConnection>,
    pub enc: Option<Arc<FieldKey>>,
    pub jwt_enc: Arc<jsonwebtoken::EncodingKey>,
    pub jwt_dec: Arc<jsonwebtoken::DecodingKey>,
    pub login_limiter: Arc<RateLimiter>,
    pub general_limiter: Arc<RateLimiter>,
    pub lockouts: Arc<Lockout>,
}

impl AppState {
    pub fn db(&self) -> Result<&DatabaseConnection, ApiError> {
        self.db
            .as_ref()
            .ok_or_else(|| ApiError::internal("database not configured"))
    }

    pub fn enc(&self) -> Result<&FieldKey, ApiError> {
        self.enc
            .as_ref()
            .map(|a| a.as_ref())
            .ok_or_else(|| ApiError::internal("FIELD_ENC_KEY not configured"))
    }
}

pub fn build_state(
    cfg: &common::config::Settings,
    db: Option<DatabaseConnection>,
    sk: &SigningKey,
) -> anyhow::Result<AppState> {
    let (enc, dec) = crypto::jwt_keys(sk)?;
    Ok(AppState {
        db,
        enc: cfg
            .field_enc_key
            .as_deref()
            .filter(|k| !k.is_empty())
            .map(crypto::field_key_from_b64)
            .transpose()?
            .map(Arc::new),
        jwt_enc: Arc::new(enc),
        jwt_dec: Arc::new(dec),
        login_limiter: Arc::new(RateLimiter::per_minute(5)),
        general_limiter: Arc::new(RateLimiter::per_minute(120)),
        lockouts: Arc::new(Lockout::appendix_c()),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    /// "access" | "temp"
    pub kind: String,
    pub iat: u64,
    pub exp: u64,
}

pub fn role_str(r: admin_user::Role) -> &'static str {
    match r {
        admin_user::Role::MotherAdmin => "mother_admin",
        admin_user::Role::Admin => "admin",
    }
}

pub fn issue_jwt(
    st: &AppState,
    sub: Uuid,
    role: &str,
    kind: &str,
    ttl: Duration,
) -> Result<String, ApiError> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
    let claims = Claims {
        sub,
        role: role.to_string(),
        kind: kind.to_string(),
        iat: now,
        exp: now + ttl.as_secs(),
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::EdDSA),
        &claims,
        &st.jwt_enc,
    )
    .map_err(|e| ApiError::internal(e.to_string()))
}

pub fn decode_jwt(st: &AppState, token: &str, want_kind: &str) -> Result<Claims, ApiError> {
    let mut v = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::EdDSA);
    v.validate_exp = true;
    let d = jsonwebtoken::decode::<Claims>(token, &st.jwt_dec, &v)
        .map_err(|_| ApiError::auth("invalid or expired token"))?;
    if d.claims.kind != want_kind {
        return Err(ApiError::auth("invalid token kind"));
    }
    Ok(d.claims)
}

pub fn cookie_value(parts: &Parts, name: &str) -> Option<String> {
    let raw = parts
        .headers
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?;
    raw.split(';').find_map(|c| {
        let c = c.trim();
        c.strip_prefix(name)
            .and_then(|r| r.strip_prefix('='))
            .map(|v| v.to_string())
            .filter(|v| !v.is_empty())
    })
}

/// Extractor: valid access JWT → active admin row.
pub struct AuthAdmin {
    pub admin: admin_user::Model,
}

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            cookie_value(parts, ACCESS_COOKIE).ok_or_else(|| ApiError::auth("no session"))?;
        let claims = decode_jwt(state, &token, "access")?;
        let db = state.db()?;
        let admin = admin_user::Entity::find_by_id(claims.sub)
            .one(db)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::auth("no session"))?;
        if admin.status == Status::Locked {
            return Err(ApiError::new(Code::Locked, "account locked"));
        }
        Ok(Self { admin })
    }
}

impl OptionalFromRequestParts<AppState> for AuthAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <AuthAdmin as FromRequestParts<AppState>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}

/// JSON body extractor that maps rejections to the Appendix A envelope.
pub struct AppJson<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for AppJson<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
    axum::Json<T>:
        axum::extract::FromRequest<S, Rejection = axum::extract::rejection::JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(v)) => Ok(Self(v)),
            Err(e) => Err(ApiError::validation(e.body_text())),
        }
    }
}

pub fn require_mother(admin: &AuthAdmin) -> Result<(), ApiError> {
    if admin.admin.role != admin_user::Role::MotherAdmin {
        return Err(ApiError::forbidden("mother_admin only"));
    }
    Ok(())
}
