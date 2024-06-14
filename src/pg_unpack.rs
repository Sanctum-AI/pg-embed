//!
//! Unpack postgresql binaries
//!
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::pg_errors::PgEmbedError;
use crate::pg_types::PgResult;

///
/// Unzip the postgresql txz file
///
/// Returns `Ok(PathBuf(txz_file_path))` file path of the txz archive on success, otherwise returns an error.
///
fn unzip_txz(zip_file_path: &PathBuf, cache_dir: &Path) -> Result<PathBuf, PgEmbedError> {
    let zip_file = File::open(zip_file_path).map_err(|e| PgEmbedError::ReadFileError {
        path: zip_file_path.clone(),
        e,
    })?;
    let mut zip_archive = ZipArchive::new(zip_file).map_err(|e| PgEmbedError::UnzipFileError {
        path: zip_file_path.clone(),
        e,
    })?;

    for i in 0..zip_archive.len() {
        let mut file = zip_archive
            .by_index(i)
            .map_err(|e| PgEmbedError::UnzipFileError {
                path: zip_file_path.clone(),
                e,
            })?;
        if file.name().ends_with(".txz") {
            let txz_path = cache_dir.join(file.name());
            let txz_file = File::create(&txz_path).map_err(|e| PgEmbedError::WriteFileError {
                path: txz_path.clone(),
                e,
            })?;
            std::io::copy(&mut file, &mut BufWriter::new(&txz_file)).map_err(|e| {
                PgEmbedError::ReadFileError {
                    path: zip_file_path.clone(),
                    e,
                }
            })?;
            return Ok(txz_path);
        }
    }
    Err(PgEmbedError::InvalidPgPackage)
}

///
/// Decompress the postgresql txz file
///
/// Returns `Ok(PathBuf(tar_file_path))` (*the file path to the postgresql tar file*) on success, otherwise returns an error.
///
fn decompress_xz(zip_file_path: &PathBuf) -> Result<PathBuf, PgEmbedError> {
    let xz_file = File::open(zip_file_path).map_err(|e| PgEmbedError::ReadFileError {
        path: zip_file_path.clone(),
        e,
    })?;
    let xz_decoder = XzDecoder::new(xz_file);
    let target_path = zip_file_path.with_extension("tar");
    let tar_file = File::create(&target_path).map_err(|e| PgEmbedError::WriteFileError {
        path: target_path.clone(),
        e,
    })?;
    std::io::copy(
        &mut BufReader::new(xz_decoder),
        &mut BufWriter::new(&tar_file),
    )
    .map_err(|e| PgEmbedError::WriteFileError {
        path: target_path.clone(),
        e,
    })?;
    Ok(target_path)
}

///
/// Unpack the postgresql tar file
///
/// Returns `Ok(())` on success, otherwise returns an error.
///
fn decompress_tar(file_path: &PathBuf, cache_dir: &PathBuf) -> Result<(), PgEmbedError> {
    let tar_file = File::open(file_path).map_err(|e| PgEmbedError::ReadFileError {
        path: file_path.clone(),
        e,
    })?;
    let mut archive = Archive::new(tar_file);
    archive
        .unpack(cache_dir)
        .map_err(PgEmbedError::UnpackFailure)?;
    Ok(())
}

///
/// Unpack the postgresql executables
///
/// Returns `Ok(())` on success, otherwise returns an error.
///
pub async fn unpack_postgres(zip_file_path: &PathBuf, cache_dir: &PathBuf) -> PgResult<()> {
    let txz_file_path = unzip_txz(zip_file_path, cache_dir)?;
    let tar_file_path = decompress_xz(&txz_file_path)?;
    std::fs::remove_file(&txz_file_path).map_err(|e| PgEmbedError::PgCleanUpFailure {
        path: txz_file_path,
        e,
    })?;
    decompress_tar(&tar_file_path, cache_dir)?;
    std::fs::remove_file(&tar_file_path).map_err(|e| PgEmbedError::PgCleanUpFailure {
        path: tar_file_path,
        e,
    })?;
    Ok(())
}
