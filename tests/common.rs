use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::time::Duration;

use env_logger::Env;

use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_errors::PgEmbedError;
use pg_embed::pg_fetch::{PG_V13, PgFetchSettings};
use pg_embed::postgres::{PgEmbed, PgSettings};

pub async fn setup(
    port: i16,
    database_dir: PathBuf,
    persistent: bool,
    migration_dir: Option<PathBuf>,
) -> Result<PgEmbed, PgEmbedError> {
    let _ =
        env_logger::Builder::from_env(
            Env::default().default_filter_or("info")
        ).is_test(true).try_init();
    let pg_settings = PgSettings {
        database_dir,
        port,
        user: "postgres".to_string(),
        password: "password".to_string(),
        auth_method: PgAuthMethod::MD5,
        persistent,
        timeout: Duration::from_secs(20),
        migration_dir,
    };
    let fetch_settings = PgFetchSettings {
        version: PG_V13,
        ..Default::default()
    };
    let mut pg = PgEmbed::new(pg_settings, fetch_settings).await?;
    pg.setup().await?;
    Ok(pg)
}

pub async fn pg_version_file_exists(db_dir: &PathBuf) -> Result<bool, PgEmbedError>{
    let mut pg_version_file = db_dir.clone();
    pg_version_file.push("PG_VERSION");
    let file_exists =
        if let Ok(_) = tokio::fs::File::open(pg_version_file.as_path()).await {
            true
        } else {
            false
        };
    Ok(file_exists)
}

pub async fn clean_up(database_dir: PathBuf, pw_file: PathBuf) -> Result<(), PgEmbedError> {
    tokio::fs::remove_dir_all(database_dir.as_path())
        .await
        .map_err(|e| PgEmbedError::PgCleanUpFailure(e))?;

    tokio::fs::remove_file(pw_file.as_path())
        .await
        .map_err(|e| PgEmbedError::PgCleanUpFailure(e))
}