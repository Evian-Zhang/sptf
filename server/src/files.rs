use crate::error::{FileError, SPTFError, UnexpectedError};
use crate::protos::sptf::{
    ListDirectoryResponse_File, ListDirectoryResponse_FileMetadata,
    ListDirectoryResponse_FileMetadata_FileType,
};
use log::{error, warn};
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

pub async fn list_dir(path: &Path) -> Result<Vec<ListDirectoryResponse_File>, Box<dyn SPTFError>> {
    let read_dir_result = fs::read_dir(path).await;
    let mut read_dir_iter = match read_dir_result {
        Ok(read_dir_iter) => read_dir_iter,
        Err(err) => {
            error!("Failed to read dir {:?}: {}", path, err);
            return Err(FileError::PermissionDenied.to_boxed_self());
        }
    };
    let mut entries = vec![];
    loop {
        let dir_entry = match read_dir_iter.next_entry().await {
            Ok(Some(dir_entry)) => dir_entry,
            Ok(None) => {
                break;
            }
            Err(err) => {
                error!("Unexpected error when listing dir {:?}: {}", path, err);
                break;
            }
        };
        let dir_entry_file_type = match dir_entry.file_type().await {
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
            ListDirectoryResponse_FileMetadata_FileType::DIRECTORY
        } else if dir_entry_file_type.is_file() {
            ListDirectoryResponse_FileMetadata_FileType::NORMAL_FILE
        } else {
            warn!(
                "File {:?} is not dir or file, so continue.",
                dir_entry.file_name()
            );
            continue;
        };
        let dir_entry_metadata = match dir_entry.metadata().await {
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
        let mut metadata = ListDirectoryResponse_FileMetadata::default();
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
        let mut entry = ListDirectoryResponse_File::default();
        let file_name = dir_entry.file_name();
        let file_name = if let Some(file_name) = file_name.to_str() {
            file_name.to_owned()
        } else {
            error!(
                "Unexpected error when converting filename {:?} to string.",
                dir_entry.file_name()
            );
            continue;
        };
        let mut file_path = path.to_path_buf();
        file_path.push(&file_name);
        let file_path_string = if let Some(file_path) = file_path.to_str() {
            file_path.to_owned()
        } else {
            error!(
                "Unexpected error when converting path {:?} to string.",
                file_path
            );
            continue;
        };
        entry.set_path(file_path_string.into());
        entry.set_file_name(file_name.into());
        entry.set_metadata(metadata);
    }

    Ok(entries)
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
