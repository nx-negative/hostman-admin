//! Environment-driven settings (Appendix B). Secrets live in `.env` only.

use serde::Deserialize;

fn default_bind() -> String {
    "127.0.0.1:8080".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database_url: Option<String>,
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default)]
    pub field_enc_key: Option<String>,
    #[serde(default)]
    pub root_key_path: Option<String>,
    #[serde(default)]
    pub admin_setup_code: Option<String>,
    #[serde(default)]
    pub admin_setup_password: Option<String>,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let s = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;
        Ok(s.try_deserialize()?)
    }
}
