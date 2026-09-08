//! Appendix A error envelope + codes.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Code {
    Validation,
    Auth,
    Totp,
    Forbidden,
    NotFound,
    Locked,
    Conflict,
    SlotLimit,
    DeviceOffline,
    NodePaused,
    MemberLimit,
    RateLimit,
    Internal,
    Upstream,
}

impl Code {
    pub fn status(self) -> StatusCode {
        match self {
            Code::Validation => StatusCode::BAD_REQUEST,
            Code::Auth | Code::Totp => StatusCode::UNAUTHORIZED,
            Code::Forbidden | Code::NodePaused => StatusCode::FORBIDDEN,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::Locked => StatusCode::LOCKED,
            Code::Conflict | Code::SlotLimit | Code::DeviceOffline | Code::MemberLimit => {
                StatusCode::CONFLICT
            }
            Code::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Upstream => StatusCode::BAD_GATEWAY,
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    pub code: Code,
    pub msg: String,
    pub details: Value,
}

impl ApiError {
    pub fn new(code: Code, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
            details: Value::Null,
        }
    }

    pub fn with_details(mut self, d: Value) -> Self {
        self.details = d;
        self
    }

    pub fn validation(m: impl Into<String>) -> Self {
        Self::new(Code::Validation, m)
    }
    pub fn auth(m: impl Into<String>) -> Self {
        Self::new(Code::Auth, m)
    }
    pub fn forbidden(m: impl Into<String>) -> Self {
        Self::new(Code::Forbidden, m)
    }
    pub fn not_found(m: impl Into<String>) -> Self {
        Self::new(Code::NotFound, m)
    }
    pub fn conflict(m: impl Into<String>) -> Self {
        Self::new(Code::Conflict, m)
    }
    pub fn internal(m: impl Into<String>) -> Self {
        Self::new(Code::Internal, m)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::internal(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.code.status(),
            Json(json!({
                "ok": false,
                "error": {"code": self.code, "msg": self.msg, "details": self.details}
            })),
        )
            .into_response()
    }
}

/// Success envelope: `{"ok": true, "data": …}`.
pub fn ok<T: Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(json!({"ok": true, "data": data}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_map_to_status() {
        assert_eq!(Code::Totp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(Code::SlotLimit.status(), StatusCode::CONFLICT);
        assert_eq!(Code::NodePaused.status(), StatusCode::FORBIDDEN);
        assert_eq!(Code::Locked.status(), StatusCode::LOCKED);
    }

    #[test]
    fn codes_serialize_screaming_snake() {
        assert_eq!(
            serde_json::to_value(Code::NodePaused).unwrap(),
            json!("NODE_PAUSED")
        );
        assert_eq!(serde_json::to_value(Code::Totp).unwrap(), json!("TOTP"));
    }
}
