use std::path::PathBuf;

use serial_test::serial;
#[cfg(feature = "sqlx_tokio")]
use sqlx_tokio::{Connection, PgConnection};

use pg_embed::pg_errors::PgEmbedError;
#[cfg(feature = "sqlx_actix")]
use sqlx_actix::{Connection, PgConnection};
#[cfg(feature = "sqlx_async_std")]
use sqlx_async_std::{Connection, PgConnection};

#[path = "common.rs"]
mod common;

#[tokio::test]
#[serial]
async fn db_create_database() -> Result<(), PgEmbedError> {
    let mut pg = common::setup(5432, PathBuf::from("data_test").join("db"), false, None).await?;
    pg.start_db().await?;
    let db_name = "test";

    pg.create_database(&db_name).await?;
    assert!(pg.database_exists(&db_name).await?);
    Ok(())
}

#[tokio::test]
#[serial]
async fn db_drop_database() -> Result<(), PgEmbedError> {
    let mut pg = common::setup(5432, PathBuf::from("data_test").join("db"), false, None).await?;
    pg.start_db().await?;
    let db_name = "test";

    pg.create_database(&db_name).await?;
    assert_eq!(true, pg.database_exists(&db_name).await?);

    pg.drop_database(&db_name).await?;
    assert_eq!(false, pg.database_exists(&db_name).await?);
    Ok(())
}

#[tokio::test]
#[serial]
async fn db_migration() -> Result<(), PgEmbedError> {
    let mut pg = common::setup(
        5432,
        PathBuf::from("data_test").join("db"),
        false,
        Some(PathBuf::from("migration_test")),
    )
        .await?;
    pg.start_db().await?;
    let db_name = "test";
    pg.create_database(&db_name).await?;

    pg.migrate(&db_name).await?;

    let db_uri = pg.full_db_uri(&db_name);

    let mut conn = PgConnection::connect(&db_uri)
        .await
        .map_err(PgEmbedError::SqlxError)?;

    let _ = sqlx_tokio::query("INSERT INTO testing (description) VALUES ('Hello')")
        .execute(&mut conn)
        .await
        .map_err(PgEmbedError::SqlxError)?;

    let rows = sqlx_tokio::query("SELECT * FROM testing")
        .fetch_all(&mut conn)
        .await
        .map_err(PgEmbedError::SqlxError)?;

    assert_eq!(1, rows.len());

    Ok(())
}
