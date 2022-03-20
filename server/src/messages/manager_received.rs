use super::session_received::RefreshFilesMessage;
use actix::prelude::*;
use std::path::PathBuf;

/// Sessions send this to Session manager
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<RefreshFilesMessage>,
}

/// Session is disconnected
///
/// Sessions send this to Session manager
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// We manually send this to Session manager
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddFilewatcher {
    pub addr: Addr<crate::filewatcher::FileWatcherActor>,
}

/// There is something chanegd in given path
///
/// Filewatcher send this to Session manager
#[derive(Message)]
#[rtype(result = "()")]
pub struct FilePathHasSomethingChanged {
    pub paths: Vec<PathBuf>,
}
