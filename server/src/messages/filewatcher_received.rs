use actix::prelude::*;

/// We manually send this to Filewatcher
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartWatchingFiles;
