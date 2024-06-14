//!
//! Cache postgresql files, access to executables, clean up files
//!

use std::cell::Cell;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::pg_enums::{OperationSystem, PgAcquisitionStatus};
use crate::pg_errors::PgEmbedError;
use crate::pg_fetch::PgFetchSettings;
use crate::pg_types::{PgCommandSync, PgResult};
use crate::pg_unpack;

lazy_static! {
    ///
    /// Stores the paths to the cache directories while acquiring the related postgres binaries
    ///
    /// Used to prevent simultaneous downloads and unpacking of the same binaries
    /// while executing multiple PgEmbed instances concurrently.
    ///
    static ref ACQUIRED_PG_BINS: Arc<Mutex<HashMap<PathBuf, PgAcquisitionStatus>>> =
    Arc::new(Mutex::new(HashMap::with_capacity(5)));
}

const PG_EMBED_CACHE_DIR_NAME: &str = "pg-embed";
const PG_VERSION_FILE_NAME: &str = "PG_VERSION";

///
/// Access to pg_ctl, initdb, database directory and cache directory
///
pub struct PgAccess {
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Database directory path
    pub database_dir: PathBuf,
    /// Postgresql pg_ctl executable path
    pub pg_ctl_exe: PathBuf,
    /// Postgresql initdb executable path
    pub init_db_exe: PathBuf,
    /// Password file path
    pub pw_file_path: PathBuf,
    /// Postgresql binaries zip file path
    pub zip_file_path: PathBuf,
    /// Postgresql database version file
    /// used for internal checks
    pg_version_file: PathBuf,
    /// Fetch settings
    fetch_settings: PgFetchSettings,
}

impl PgAccess {
    ///
    /// Create a new instance
    ///
    /// Directory structure for cached postgresql binaries will be created
    ///
    pub async fn new(
        fetch_settings: &PgFetchSettings,
        database_dir: &PathBuf,
        cache_dir: Option<&PathBuf>,
    ) -> Result<Self, PgEmbedError> {
        let cache_dir = match cache_dir {
            Some(d) => {
                std::fs::create_dir_all(d)
                    .map_err(|e| PgEmbedError::DirCreationError { dir: d.clone(), e })?;
                d.clone()
            }
            None => Self::create_cache_dir_structure(fetch_settings)?,
        };

        Self::create_db_dir_structure(database_dir)?;
        // pg_ctl executable
        let pg_ctl = cache_dir.clone().join("bin").join("pg_ctl");
        // initdb executable
        let init_db = cache_dir.clone().join("bin").join("initdb");
        // postgres zip file
        let mut zip_file_path = cache_dir.clone();
        let platform = fetch_settings.platform();
        let file_name = format!("{}-{}.zip", platform, &fetch_settings.version.0);
        zip_file_path.push(file_name);
        // password file
        let mut pw_file = database_dir.clone();
        pw_file.set_extension("pwfile");
        // postgres version file
        let mut pg_version_file = database_dir.clone();
        pg_version_file.push(PG_VERSION_FILE_NAME);

        Ok(PgAccess {
            cache_dir,
            database_dir: database_dir.clone(),
            pg_ctl_exe: pg_ctl,
            init_db_exe: init_db,
            pw_file_path: pw_file,
            zip_file_path,
            pg_version_file,
            fetch_settings: fetch_settings.clone(),
        })
    }

    ///
    /// Create directory structure for cached postgresql executables
    ///
    /// Returns PathBuf(cache_directory) on success, an error otherwise
    ///
    fn create_cache_dir_structure(fetch_settings: &PgFetchSettings) -> PgResult<PathBuf> {
        let cache_dir = dirs::cache_dir().ok_or_else(|| PgEmbedError::NoSystemCacheDirectory)?;
        let os_string = match fetch_settings.operating_system {
            OperationSystem::Darwin | OperationSystem::Windows | OperationSystem::Linux => {
                fetch_settings.operating_system.to_string()
            }
            OperationSystem::AlpineLinux => {
                format!("arch_{}", fetch_settings.operating_system)
            }
        };
        let pg_path = format!(
            "{}/{}/{}/{}",
            PG_EMBED_CACHE_DIR_NAME,
            os_string,
            fetch_settings.architecture,
            fetch_settings.version.0
        );
        let mut cache_pg_embed = cache_dir.clone();
        cache_pg_embed.push(pg_path);
        std::fs::create_dir_all(&cache_pg_embed).map_err(|e| PgEmbedError::DirCreationError {
            dir: cache_pg_embed.clone(),
            e,
        })?;
        Ok(cache_pg_embed)
    }

    fn create_db_dir_structure(db_dir: &PathBuf) -> PgResult<()> {
        std::fs::create_dir_all(db_dir).map_err(|e| PgEmbedError::DirCreationError {
            dir: db_dir.clone(),
            e,
        })?;
        Ok(())
    }

    ///
    /// Download and unpack postgres binaries
    ///
    pub async fn maybe_acquire_postgres(&self) -> PgResult<()> {
        let mut lock = ACQUIRED_PG_BINS.lock().await;

        if self.pg_executables_cached()? {
            return Ok(());
        }

        lock.insert(self.cache_dir.clone(), PgAcquisitionStatus::InProgress);
        let pg_bin_data = self.fetch_settings.fetch_postgres().await?;
        self.write_pg_zip(&pg_bin_data)?;
        log::debug!(
            "Unpacking postgres binaries {} {}",
            self.zip_file_path.display(),
            self.cache_dir.display()
        );
        pg_unpack::unpack_postgres(&self.zip_file_path, &self.cache_dir).await?;
        std::fs::remove_file(&self.zip_file_path).map_err(|e| PgEmbedError::PgCleanUpFailure {
            path: self.zip_file_path.clone(),
            e,
        })?;

        lock.insert(self.cache_dir.clone(), PgAcquisitionStatus::Finished);
        Ok(())
    }

    ///
    /// Check if postgresql executables are already cached
    ///
    pub fn pg_executables_cached(&self) -> PgResult<bool> {
        Self::path_exists(self.init_db_exe.as_path())
    }

    ///
    /// Check if database files exist
    ///
    pub async fn db_files_exist(&self) -> PgResult<bool> {
        Ok(self.pg_executables_cached()? && Self::path_exists(self.pg_version_file.as_path())?)
    }

    ///
    /// Check if database version file exists
    ///
    pub async fn pg_version_file_exists(db_dir: &Path) -> PgResult<bool> {
        let mut pg_version_file = db_dir.to_path_buf();
        pg_version_file.push(PG_VERSION_FILE_NAME);
        let file_exists = std::fs::File::open(pg_version_file.as_path()).is_ok();
        Ok(file_exists)
    }

    ///
    /// Check if file path exists
    ///
    fn path_exists(file: &Path) -> PgResult<bool> {
        if std::fs::File::open(file).is_ok() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    ///
    /// Check postgresql acquisition status
    ///
    pub async fn acquisition_status(&self) -> PgAcquisitionStatus {
        let lock = ACQUIRED_PG_BINS.lock().await;
        let acquisition_status = lock.get(&self.cache_dir);
        match acquisition_status {
            None => PgAcquisitionStatus::Undefined,
            Some(status) => *status,
        }
    }

    ///
    /// Write pg binaries zip to postgresql cache directory
    ///
    fn write_pg_zip(&self, bytes: &[u8]) -> PgResult<()> {
        let mut file = std::fs::File::create(self.zip_file_path.as_path()).map_err(|e| {
            PgEmbedError::WriteFileError {
                path: self.zip_file_path.clone(),
                e,
            }
        })?;
        file.write(bytes)
            .map_err(|e| PgEmbedError::WriteFileError {
                path: self.zip_file_path.clone(),
                e,
            })?;
        Ok(())
    }

    ///
    /// Clean up created files and directories.
    ///
    /// Remove created directories containing the database and the password file.
    ///
    pub fn clean(&self) -> PgResult<()> {
        // not using tokio::fs async methods because clean() is called on drop
        std::fs::remove_dir_all(self.database_dir.as_path()).map_err(|e| {
            PgEmbedError::PgCleanUpFailure {
                path: self.database_dir.clone(),
                e,
            }
        })?;
        std::fs::remove_file(self.pw_file_path.as_path()).map_err(|e| {
            PgEmbedError::PgCleanUpFailure {
                path: self.pw_file_path.clone(),
                e,
            }
        })?;
        Ok(())
    }

    ///
    /// Purge postgresql executables
    ///
    /// Remove all cached postgresql executables
    ///
    pub fn purge(cache_dir: &Path) -> PgResult<()> {
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).map_err(|e| PgEmbedError::PgCleanUpFailure {
                path: cache_dir.to_path_buf(),
                e,
            })?;
        }
        Ok(())
    }

    ///
    /// Clean up database directory and password file
    ///
    pub async fn clean_up(database_dir: PathBuf, pw_file: PathBuf) -> PgResult<()> {
        std::fs::remove_dir_all(&database_dir).map_err(|e| PgEmbedError::PgCleanUpFailure {
            path: database_dir,
            e,
        })?;

        std::fs::remove_file(&pw_file)
            .map_err(|e| PgEmbedError::PgCleanUpFailure { path: pw_file, e })
    }

    ///
    /// Create a database password file
    ///
    /// Returns `Ok(())` on success, otherwise returns an error.
    ///
    pub fn create_password_file(&self, password: &[u8]) -> PgResult<()> {
        let mut file = std::fs::File::create(self.pw_file_path.as_path()).map_err(|e| {
            PgEmbedError::WriteFileError {
                path: self.pw_file_path.clone(),
                e,
            }
        })?;
        file.write(password)
            .map_err(|e| PgEmbedError::WriteFileError {
                path: self.pw_file_path.clone(),
                e,
            })?;
        Ok(())
    }

    ///
    /// Create synchronous pg_ctl stop command
    ///
    pub fn stop_db_command_sync(&self, database_dir: &Path) -> PgCommandSync {
        let pg_ctl_executable = self.pg_ctl_exe.to_str().unwrap();
        let mut command = Box::new(Cell::new(std::process::Command::new(pg_ctl_executable)));
        command
            .get_mut()
            .args(["stop", "-w", "-D", database_dir.to_str().unwrap()]);
        command
    }
}
