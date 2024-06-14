//!
//! Enums
//!

use std::error::Error;

use crate::command_executor::ProcessStatus;
use crate::pg_errors::PgEmbedError;

///
/// Postgresql authentication method
///
/// Choose between plain password, md5 or scram_sha_256 authentication.
/// Scram_sha_256 authentication is only available on postgresql versions >= 11
///
pub enum PgAuthMethod {
    /// plain-text
    Plain,
    /// md5
    MD5,
    /// scram_sha_256
    ScramSha256,
}

///
/// Postgresql server status
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PgServerStatus {
    /// Postgres uninitialized
    Uninitialized,
    /// Initialization process running
    Initializing,
    /// Initialization process finished
    Initialized,
    /// Postgres server process starting
    Starting,
    /// Postgres server process started
    Started,
    /// Postgres server process stopping
    Stopping,
    /// Postgres server process stopped
    Stopped,
    /// Postgres failure
    Failure,
}

///
/// Postgesql process type
///
/// Used internally for distinguishing processes being executed
///
pub enum PgProcessType {
    /// initdb process
    InitDb,
    /// pg_ctl start process
    StartDb,
    /// pg_ctl stop process
    StopDb,
}

impl ProcessStatus<PgServerStatus, PgEmbedError> for PgProcessType {
    fn status_entry(&self) -> PgServerStatus {
        match self {
            PgProcessType::InitDb => PgServerStatus::Initializing,
            PgProcessType::StartDb => PgServerStatus::Starting,
            PgProcessType::StopDb => PgServerStatus::Stopping,
        }
    }

    fn status_exit(&self) -> PgServerStatus {
        match self {
            PgProcessType::InitDb => PgServerStatus::Initialized,
            PgProcessType::StartDb => PgServerStatus::Started,
            PgProcessType::StopDb => PgServerStatus::Stopped,
        }
    }

    fn error_type(&self) -> PgEmbedError {
        match self {
            PgProcessType::InitDb => PgEmbedError::PgInitFailure,
            PgProcessType::StartDb => PgEmbedError::PgStartFailure,
            PgProcessType::StopDb => PgEmbedError::PgStopFailure,
        }
    }

    fn wrap_error<E: Error + Sync + Send + 'static>(
        &self,
        error: E,
        message: String,
    ) -> PgEmbedError {
        PgEmbedError::PgError {
            source: Box::new(error),
            message,
        }
    }
}

impl std::fmt::Display for PgProcessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgProcessType::InitDb => write!(f, "initdb"),
            PgProcessType::StartDb => write!(f, "start"),
            PgProcessType::StopDb => write!(f, "stop"),
        }
    }
}

/// The operation systems enum
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum OperationSystem {
    Darwin,
    Windows,
    Linux,
    AlpineLinux,
}

impl std::fmt::Display for OperationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OperationSystem::Darwin => "darwin",
                OperationSystem::Windows => "windows",
                OperationSystem::Linux => "linux",
                OperationSystem::AlpineLinux => "linux",
            }
        )
    }
}

impl Default for OperationSystem {
    fn default() -> Self {
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            OperationSystem::Darwin
        }

        #[cfg(target_os = "linux")]
        {
            OperationSystem::Linux
        }

        #[cfg(target_os = "windows")]
        {
            OperationSystem::Windows
        }
    }
}

/// The cpu architectures enum
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Architecture {
    Amd64,
    I386,
    Arm32v6,
    Arm32v7,
    Arm64v8,
    Ppc64le,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match &self {
            Architecture::Amd64 => "amd64",
            Architecture::I386 => "i386",
            Architecture::Arm32v6 => "arm32v6",
            Architecture::Arm32v7 => "arm32v7",
            Architecture::Arm64v8 => "arm64v8",
            Architecture::Ppc64le => "ppc64le",
        };
        write!(f, "{s}")
    }
}

impl Default for Architecture {
    fn default() -> Self {
        #[cfg(not(any(
            target_arch = "x86",
            target_arch = "arm",
            target_arch = "aarch64",
            target_arch = "powerpc64"
        )))]
        {
            Architecture::Amd64
        }

        #[cfg(target_arch = "x86")]
        {
            Architecture::I386
        }

        #[cfg(target_arch = "arm")]
        {
            Architecture::Arm32v7
        }

        #[cfg(target_arch = "aarch64")]
        {
            Architecture::Arm64v8
        }

        #[cfg(target_arch = "powerpc64")]
        {
            Architecture::Ppc64le
        }
    }
}

/// The postgresql binaries acquisition status
#[derive(Copy, Clone, PartialEq)]
pub enum PgAcquisitionStatus {
    /// Acquiring postgresql binaries
    InProgress,
    /// Finished acquiring postgresql binaries
    Finished,
    /// No acquisition
    Undefined,
}
