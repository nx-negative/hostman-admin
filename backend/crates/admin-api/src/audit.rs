//! Audit log writer (P6: silent on failure).

use domain::entity::audit_log;
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use serde_json::Value;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn write(
    db: &DatabaseConnection,
    actor: Option<Uuid>,
    action: &str,
    target_type: Option<&str>,
    target_id: Option<&str>,
    meta: Value,
    ip: Option<String>,
) {
    let row = audit_log::ActiveModel {
        id: Set(Uuid::now_v7()),
        actor_admin_id: Set(actor),
        action: Set(action.to_string()),
        target_type: Set(target_type.map(str::to_string)),
        target_id: Set(target_id.map(str::to_string)),
        meta: Set((!meta.is_null()).then_some(meta)),
        ip: Set(ip),
        at: Set(time::OffsetDateTime::now_utc()),
    };
    if let Err(e) = audit_log::Entity::insert(row).exec(db).await {
        tracing::warn!(error = %e, "audit write failed");
    }
}
