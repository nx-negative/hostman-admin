//! Health routes + middleware stack (§8 Health).

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Option<DatabaseConnection>,
}

pub fn router(st: AppState) -> Router {
    let health = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));
    Router::new()
        .merge(health.clone())
        .nest("/api/v1", health)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive()) // dev-only; locked down in P8
        .layer(TraceLayer::new_for_http())
        .with_state(st)
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
        common::error::ok(json!({"status": "ok", "db": "ok"}))
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

    #[tokio::test]
    async fn healthz_ok() {
        let (status, body) = get(router(AppState { db: None }), "/healthz").await;
        assert_eq!(status, 200);
        assert_eq!(body["ok"], json!(true));
        assert_eq!(body["data"]["status"], json!("ok"));
    }

    #[tokio::test]
    async fn readyz_ok_nested_under_api_v1() {
        let (status, body) = get(router(AppState { db: None }), "/api/v1/healthz").await;
        assert_eq!(status, 200);
        assert_eq!(body["data"]["status"], json!("ok"));
    }

    #[tokio::test]
    async fn readyz_down_without_db() {
        let (status, body) = get(router(AppState { db: None }), "/api/v1/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], json!("UPSTREAM"));
    }
}
