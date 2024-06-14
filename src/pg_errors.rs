//!
//! Errors
//!

use std::path::PathBuf;
use thiserror::Error;
use zip::result::ZipError;


///
/// Common pg_embed errors, independent of features used
///
#[derive(Error, Debug)]
pub enum PgEmbedError {
    /// Invalid postgresql binaries download url
    #[error("System does not have standard cache directory")]
    NoSystemCacheDirectory,
    #[error("Invalid postgresql binaries package")]
    InvalidPgPackage,
    #[error("Could not write file: {path} due to error {e}")]
    WriteFileError { e: std::io::Error, path: PathBuf },
    #[error("Failed to unzip: {path} due to error {e}")]
    UnzipFileError { e: ZipError, path: PathBuf },
    #[error("Could not read file: {path} due to error {e}")]
    ReadFileError{ e: std::io::Error, path: PathBuf },
    /// Could not create directory
    #[error("Failed to create directory: {dir} error: {e}")]
    DirCreationError { dir: PathBuf, e: std::io::Error },
    /// Failed to unpack postgresql binaries
    #[error("Failed to unpack postgresql binaries: {0}")]
    UnpackFailure(#[from] std::io::Error),
    #[error("Postgresql could not be started")]
    PgStartFailure,
    #[error("Postgresql could not be stopped")]
    PgStopFailure,
    /// Postgresql could not be initialized
    #[error("Failed to initialize postgres database")]
    PgInitFailure,
    /// Clean up error
    #[error("Failed to remove {path} due to {e}")]
    PgCleanUpFailure { e: std::io::Error, path: PathBuf },
    /// Task join error
    #[error("{message} due to error: {source}")]
    PgError {
        source: Box<dyn std::error::Error + Sync + Send + 'static>,
        message: String,
    },
    #[error("Download failure: {0}")]
    DownloadFailure(#[from] reqwest::Error),
    #[error("Sqlx query error: {0}")]
    SqlxError(#[from] sqlx_tokio::error::Error),
    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx_tokio::migrate::MigrateError),
}
