mod admins;
mod audit;
mod auth;
mod http;
mod state;

use common::config::Settings;
use common::crypto::{
    encrypt_field, ensure_root_key, gen_field_key_b64, gen_login_code, gen_recovery_code,
    hash_secret, normalize_code, valid_code,
};
use domain::entity::admin_user;
use domain::migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, EntityTrait, PaginatorTrait, Set};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::logging::init();
    let cfg = Settings::load()?;
    match std::env::args().nth(1).as_deref() {
        Some("setup") => setup(cfg).await,
        _ => serve(cfg).await,
    }
}

async fn connect(cfg: &Settings) -> anyhow::Result<DatabaseConnection> {
    let url = cfg
        .database_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL not set (.env)"))?;
    Ok(Database::connect(url).await?)
}

/// §13 step 4: root key + FIELD_ENC_KEY (auto-gen into .env) → migrations → mother-admin bootstrap.
async fn setup(cfg: Settings) -> anyhow::Result<()> {
    let key_path = key_path(&cfg)?;
    ensure_root_key(key_path.to_string_lossy().as_ref())?;
    println!("root key ready: {}", key_path.display());

    let mut cfg = cfg;
    if cfg
        .field_enc_key
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        upsert_env("FIELD_ENC_KEY", &gen_field_key_b64())?;
        cfg = Settings::load()?;
        println!("FIELD_ENC_KEY generated → .env");
    }

    let db = connect(&cfg).await?;
    Migrator::up(&db, None).await?;
    println!("migrations applied");
    bootstrap(&cfg, &db).await?;
    Ok(())
}

/// ROOT_KEY_PATH is anchored to the repo root (the dir holding .env).
fn key_path(cfg: &Settings) -> anyhow::Result<std::path::PathBuf> {
    let rel = cfg
        .root_key_path
        .clone()
        .unwrap_or_else(|| "backend/keys/root.ed25519".into());
    let p = std::path::PathBuf::from(&rel);
    if p.is_absolute() {
        return Ok(p);
    }
    let env_dir = dotenvy::dotenv()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    Ok(env_dir.join(p))
}

async fn bootstrap(cfg: &Settings, db: &DatabaseConnection) -> anyhow::Result<()> {
    let enc_key = common::crypto::field_key_from_b64(
        cfg.field_enc_key
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("FIELD_ENC_KEY missing"))?,
    )?;
    if admin_user::Entity::find().count(db).await? > 0 {
        println!("admin_users not empty — bootstrap skipped");
        return Ok(());
    }
    let code = match cfg.admin_setup_code.as_deref().filter(|s| !s.is_empty()) {
        Some(c) => {
            let n = normalize_code(c);
            anyhow::ensure!(
                valid_code(&n),
                "ADMIN_SETUP_CODE must be 32 Crockford chars"
            );
            n
        }
        None => gen_login_code(),
    };
    let password = match cfg
        .admin_setup_password
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        Some(p) => p.to_string(),
        None => gen_recovery_code(),
    };
    let now = time::OffsetDateTime::now_utc();
    let row = admin_user::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        login_code_prefix: Set(code.chars().take(4).collect()),
        login_code_hash: Set(encrypt_field(&enc_key, &hash_secret(&code)?)?),
        password_hash: Set(hash_secret(&password)?),
        email: Set(None),
        totp_secret: Set(None),
        totp_enabled: Set(false),
        role: Set(admin_user::Role::MotherAdmin),
        status: Set(admin_user::Status::Active),
        created_at: Set(now),
        updated_at: Set(now),
    };
    admin_user::Entity::insert(row).exec(db).await?;
    println!("=== MOTHER ADMIN (shown once, delete after saving) ===");
    println!("login_code: {code}");
    println!("password:   {password}");
    println!("=====================================================");
    Ok(())
}

/// Add/replace a `KEY=VALUE` line in the .env dotenvy resolves.
fn upsert_env(key: &str, val: &str) -> anyhow::Result<()> {
    let path = dotenvy::dotenv().unwrap_or_else(|_| std::path::PathBuf::from(".env"));
    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)?
            .lines()
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };
    let new_line = format!("{key}={val}");
    match lines.iter_mut().find(|l| l.split('=').next() == Some(key)) {
        Some(l) => *l = new_line,
        None => lines.push(new_line),
    }
    std::fs::write(&path, lines.join("\n") + "\n")?;
    Ok(())
}

async fn serve(cfg: Settings) -> anyhow::Result<()> {
    let sk = ensure_root_key(&key_path(&cfg)?.to_string_lossy())?;
    let db = match connect(&cfg).await {
        Ok(db) => Some(db),
        Err(e) => {
            tracing::warn!(error = %e, "db unavailable — serving degraded");
            None
        }
    };
    let st = state::build_state(&cfg, db, &sk)?;
    let app = http::router(st);
    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!(bind = %cfg.bind, "listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
