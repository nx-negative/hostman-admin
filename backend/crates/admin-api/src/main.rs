mod http;

use common::config::Settings;
use domain::migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

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

/// Runs migrations on the cloud DB (§13 step 4; keygen + bootstrap arrive in P1).
async fn setup(cfg: Settings) -> anyhow::Result<()> {
    let db = connect(&cfg).await?;
    Migrator::up(&db, None).await?;
    println!("migrations applied");
    Ok(())
}

async fn serve(cfg: Settings) -> anyhow::Result<()> {
    let db = match connect(&cfg).await {
        Ok(db) => Some(db),
        Err(e) => {
            tracing::warn!(error = %e, "db unavailable — serving degraded");
            None
        }
    };
    let app = http::router(http::AppState { db });
    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!(bind = %cfg.bind, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}
