use crate::error::{FileError, SPTFError, UnexpectedError};
use crate::protos::sptf::{
    DirectoryLayout, DirectoryLayout_File, DirectoryLayout_FileMetadata,
    DirectoryLayout_FileMetadata_FileType, FileUploadRequest, ListDirectoryResponse,
};
use flate2::{write::GzEncoder, Compression};
use log::{error, warn};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

/// Compose root path and user-aware path
///
/// Don't worry about attacker gives something like "../", it is restricted to access
fn real_path(root_path: &Path, path: &Path) -> PathBuf {
    let path = if let Ok(path) = path.strip_prefix("/") {
        path.to_path_buf()
    } else {
        path.to_path_buf()
    };
    let mut real_path = root_path.to_path_buf();
    real_path.push(path);
    real_path
}

/// Extract user-aware path from root path and real path
fn user_aware_path(root_path: &Path, real_path: &Path) -> Option<PathBuf> {
    real_path
        .strip_prefix(root_path)
        .ok()
        .map(Path::to_path_buf)
}

pub fn list_dir(root_path: &Path, user_aware_path: &Path) -> ListDirectoryResponse {
    let real_path = real_path(root_path, user_aware_path);
    let mut list_directory_response = ListDirectoryResponse::default();

    list_directory_response.set_directory_path((*real_path.to_string_lossy()).into());
    match list_dir_internal(&root_path, &real_path) {
        Ok(directory_layout) => {
            list_directory_response.set_DirectoryLayout(directory_layout);
        }
        Err(error) => {
            list_directory_response.set_ErrorResponse(error.to_proto_error());
        }
    }

    list_directory_response
}

fn list_dir_internal(
    root_path: &Path,
    real_path: &Path,
) -> Result<DirectoryLayout, Box<dyn SPTFError>> {
    let read_dir_result = fs::read_dir(real_path);
    let mut read_dir_iter = match read_dir_result {
        Ok(read_dir_iter) => read_dir_iter,
        Err(err) => {
            error!("Failed to read dir {:?}: {}", real_path, err);
            return Err(FileError::PermissionDenied.to_boxed_self());
        }
    };
    let mut entries = vec![];
    loop {
        let dir_entry = match read_dir_iter.next() {
            Some(Ok(dir_entry)) => dir_entry,
            None => {
                break;
            }
            Some(Err(err)) => {
                error!("Unexpected error when listing dir {:?}: {}", real_path, err);
                break;
            }
        };
        let dir_entry_file_type = match dir_entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                error!(
                    "Unexpected error when retrieving file type of file {:?}: {}",
                    dir_entry.file_name(),
                    err
                );
                continue;
            }
        };
        let file_type = if dir_entry_file_type.is_dir() {
            DirectoryLayout_FileMetadata_FileType::DIRECTORY
        } else if dir_entry_file_type.is_file() {
            DirectoryLayout_FileMetadata_FileType::NORMAL_FILE
        } else {
            warn!(
                "File {:?} is not dir or file, so continue.",
                dir_entry.file_name()
            );
            continue;
        };
        let dir_entry_metadata = match dir_entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                error!(
                    "Unexpected error when retrieving metadata of file {:?}: {}",
                    dir_entry.file_name(),
                    err
                );
                continue;
            }
        };
        let mut metadata = DirectoryLayout_FileMetadata::default();
        metadata.set_file_type(file_type);
        metadata.set_size(dir_entry_metadata.len());
        let modified_timestamp =
            if let Ok(timestamp) = retrieve_timestamp(dir_entry_metadata.modified()) {
                timestamp
            } else {
                continue;
            };
        let accessed_timestamp =
            if let Ok(timestamp) = retrieve_timestamp(dir_entry_metadata.accessed()) {
                timestamp
            } else {
                continue;
            };
        let created_timestamp =
            if let Ok(timestamp) = retrieve_timestamp(dir_entry_metadata.created()) {
                timestamp
            } else {
                continue;
            };
        metadata.set_modified_timestamp(modified_timestamp);
        metadata.set_accessed_timestamp(accessed_timestamp);
        metadata.set_created_timestamp(created_timestamp);
        let mut entry = DirectoryLayout_File::default();
        let file_name = dir_entry.file_name().to_string_lossy().to_string();
        let mut file_path = real_path.to_path_buf();
        file_path.push(&file_name);
        let file_path = if let Some(file_path) = user_aware_path(&root_path, &file_path) {
            file_path
        } else {
            error!("Unexpected error here: cannot convert real path to user aware path");
            return Err(UnexpectedError.to_boxed_self());
        };
        entry.set_path((*file_path.to_string_lossy()).into());
        entry.set_file_name(file_name.into());
        entry.set_metadata(metadata);
        entries.push(entry);
    }

    let mut directory_layout = DirectoryLayout::default();
    directory_layout.set_files(entries.into());
    Ok(directory_layout)
}

fn retrieve_timestamp(
    system_time_result: io::Result<SystemTime>,
) -> Result<u64, Box<dyn SPTFError>> {
    let system_time = match system_time_result {
        Ok(system_time) => system_time,
        Err(err) => {
            error!("Unexpected error when retrieving system time: {}", err);
            return Err(UnexpectedError.to_boxed_self());
        }
    };
    let duration = match system_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(err) => {
            error!("Unexpected error when retrieving system time: {}", err);
            return Err(UnexpectedError.to_boxed_self());
        }
    };
    let timestamp = duration.as_secs();
    Ok(timestamp)
}

pub async fn compress_files(
    root_path: &Path,
    files: &Vec<String>,
) -> Result<File, Box<dyn SPTFError>> {
    let temp_dir = match TempDir::new() {
        Ok(temp_dir) => temp_dir,
        Err(err) => {
            error!("Failed to create temp dir: {}", err);
            return Err(UnexpectedError.to_boxed_self());
        }
    };
    let temp_compressed_file = match tempfile::tempfile() {
        Ok(temp_compressed_file) => temp_compressed_file,
        Err(err) => {
            error!("Failed to create temp file: {}", err);
            return Err(UnexpectedError.to_boxed_self());
        }
    };
    for file in files {
        let user_aware_file_path = PathBuf::from(file);
        let file_name = if let Some(file_name) = user_aware_file_path.file_name() {
            file_name
        } else {
            error!("Failed to extract file name of {:?}", user_aware_file_path);
            continue;
        };
        let mut temp_file = temp_dir.path().to_path_buf();
        temp_file.push(file_name);
        let real_file_path = real_path(&root_path, &user_aware_file_path);
        if let Err(err) = tokio::fs::copy(&real_file_path, temp_file).await {
            error!("Failed to copy {:?}: {}", real_file_path, err);
            return Err(FileError::PermissionDenied.to_boxed_self());
        }
    }

    let enc = GzEncoder::new(&temp_compressed_file, Compression::default());
    let mut tar = tar::Builder::new(enc);
    if let Err(err) = tar.append_dir_all("target", temp_dir.path()) {
        error!("Failed to add dirs to tar: {}", err);
        return Err(UnexpectedError.to_boxed_self());
    }
    if let Err(err) = tar.finish() {
        error!("Failed to finish tar: {}", err);
        return Err(UnexpectedError.to_boxed_self());
    }
    drop(tar);

    Ok(temp_compressed_file)
}

pub async fn upload_files(
    root_path: &Path,
    file_upload_request: FileUploadRequest,
) -> Result<(), Box<dyn SPTFError>> {
    let dir_path = file_upload_request.get_dir_path();
    let mut result = Ok(());
    for file in file_upload_request.get_uploaded_file() {
        let file_name = file.get_file_name();
        let mut user_aware_file_path = PathBuf::from(&dir_path);
        user_aware_file_path.push(file_name);
        let real_file_path = real_path(&root_path, &user_aware_file_path);
        let content = file.get_content();
        if let Err(err) = tokio::fs::write(&real_file_path, content).await {
            error!("Failed to write to {:?}: {}", real_file_path, err);
            result = Err(FileError::PermissionDenied.to_boxed_self());
            continue;
        }
    }

    result
}
