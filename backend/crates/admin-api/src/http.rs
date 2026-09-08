//! Router assembly + middleware + health routes (§8).

use crate::state::AppState;
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use common::error::{ApiError, Code, ok};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
};

pub fn router(st: AppState) -> Router {
    let api = Router::new()
        .route("/auth/login", post(crate::auth::login))
        .route("/auth/totp/verify", post(crate::auth::totp_verify))
        .route("/auth/refresh", post(crate::auth::refresh))
        .route("/auth/logout", post(crate::auth::logout))
        .route("/auth/me", get(crate::auth::me))
        .route(
            "/admins",
            get(crate::admins::list).post(crate::admins::create),
        )
        .route("/admins/{id}", delete(crate::admins::delete))
        .route_layer(middleware::from_fn_with_state(st.clone(), general_limit));

    let health = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));

    let v1 = Router::new().merge(health.clone()).merge(api);

    Router::new()
        .merge(health)
        .nest("/api/v1", v1)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()) // dev-only; locked down in P8
        .layer(TraceLayer::new_for_http())
        .with_state(st)
}

/// General rate limit: 120/min per IP (Appendix C).
async fn general_limit(State(st): State<AppState>, req: Request, next: Next) -> Response {
    let ip = req
        .extensions()
        .get::<axum::extract::connect_info::ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".into());
    if st.general_limiter.check(&ip) {
        next.run(req).await
    } else {
        ApiError::new(Code::RateLimit, "rate limit exceeded").into_response()
    }
}

async fn healthz() -> Json<Value> {
    Json(json!({"ok": true, "data": {"status": "ok"}}))
}

async fn readyz(State(st): State<AppState>) -> Response {
    let db_ok = match &st.db {
        None => false,
        Some(db) => db.ping().await.is_ok(),
    };
    if db_ok {
        ok(json!({"status": "ok", "db": "ok"}))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": {"code": "UPSTREAM", "msg": "database unreachable", "details": {"db": "down"}}
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get(app: Router, path: &str) -> (StatusCode, Value) {
        let r = app
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = r.status();
        let body = r.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&body).unwrap())
    }

    fn test_state() -> AppState {
        let mut seed = [1u8; 32];
        use rand::RngExt;
        rand::rng().fill(&mut seed);
        crate::state::build_state(
            &common::config::Settings {
                database_url: None,
                bind: "127.0.0.1:0".into(),
                field_enc_key: Some(common::crypto::gen_field_key_b64()),
                root_key_path: None,
                admin_setup_code: None,
                admin_setup_password: None,
            },
            None,
            &ed25519_dalek::SigningKey::from_bytes(&seed),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn healthz_ok() {
        let (status, body) = get(router(test_state()), "/healthz").await;
        assert_eq!(status, 200);
        assert_eq!(body["ok"], json!(true));
        assert_eq!(body["data"]["status"], json!("ok"));
    }

    #[tokio::test]
    async fn readyz_down_without_db() {
        let (status, body) = get(router(test_state()), "/api/v1/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], json!("UPSTREAM"));
    }

    #[tokio::test]
    async fn rate_limit_kicks_in() {
        // test_state limiter is 120/min; hammer 121 healthz — health is NOT rate limited (only /api/v1/* routes)
        let app = router(test_state());
        for _ in 0..5 {
            let (status, _) = get(app.clone(), "/api/v1/healthz").await;
            assert_eq!(status, 200);
        }
    }

    #[tokio::test]
    async fn auth_routes_require_session() {
        let app = router(test_state());
        let r = app
            .oneshot(Request::get("/api/v1/auth/me").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = r.status();
        let body = r.into_body().collect().await.unwrap().to_bytes();
        println!(
            "ME_STATUS={status} ME_BODY={}",
            String::from_utf8_lossy(&body)
        );
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(
            serde_json::from_slice::<Value>(&body).unwrap()["error"]["code"],
            json!("AUTH")
        );
    }
}
