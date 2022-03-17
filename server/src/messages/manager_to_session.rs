use actix::prelude::*;
use std::path::PathBuf;

/// Session manager sends this to Sessions
#[derive(Message)]
#[rtype(result = "()")]
pub struct RefreshFilesMessage {
    pub file_paths: Vec<PathBuf>,
}
