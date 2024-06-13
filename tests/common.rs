use std::path::PathBuf;
use std::time::Duration;

use env_logger::Env;
use futures::TryFutureExt;

use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_errors::{PgEmbedError, PgEmbedErrorType};
use pg_embed::pg_fetch::{PgFetchSettings, PG_V16};
use pg_embed::postgres::{PgEmbed, PgSettings};

pub async fn setup(
    port: u16,
    database_dir: PathBuf,
    persistent: bool,
    migration_dir: Option<PathBuf>,
) -> Result<PgEmbed, PgEmbedError> {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .is_test(true)
        .try_init();
    let cache_dir = PathBuf::from("data_test").join("cache");
    tokio::fs::create_dir_all(&cache_dir)
        .map_err(|e| PgEmbedError {
            error_type: PgEmbedErrorType::DirCreationError,
            source: Some(Box::new(e)),
            message: None,
        })
        .await?;
    let pg_settings = PgSettings {
        database_dir,
        cache_dir: Some(cache_dir),
        port,
        user: "postgres".to_string(),
        password: "password".to_string(),
        auth_method: PgAuthMethod::MD5,
        persistent,
        timeout: Some(Duration::from_secs(10)),
        migration_dir,
    };
    let fetch_settings = PgFetchSettings {
        version: PG_V16,
        ..Default::default()
    };
    let mut pg = PgEmbed::new(pg_settings, fetch_settings).await?;
    pg.setup().await?;
    Ok(pg)
}
