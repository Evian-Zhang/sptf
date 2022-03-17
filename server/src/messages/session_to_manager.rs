use super::manager_to_session::RefreshFilesMessage;
use actix::prelude::*;

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
