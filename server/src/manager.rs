use crate::filewatcher::FileWatcherActor;
use crate::messages::*;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::HashMap;

/// Session manager
pub struct SessionManager {
    sessions: HashMap<usize, Recipient<RefreshFilesMessage>>,
    /// Use this field to hold an address to filewatcher in case it is
    /// stopped due to all addresses to it get dropped
    _filewatcher_addr: Option<Addr<FileWatcherActor>>,
    /// Random number generator to generate session id, be thread_rng by default
    rng: ThreadRng,
}

impl SessionManager {
    /// Create a new user session
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            _filewatcher_addr: None,
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;
}

impl Handler<Connect> for SessionManager {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<FilePathHasSomethingChanged> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: FilePathHasSomethingChanged, _: &mut Context<Self>) -> Self::Result {
        self.sessions.values().for_each(|session_recipient| {
            session_recipient.do_send(RefreshFilesMessage {
                file_paths: msg.paths.clone(),
            })
        })
    }
}

impl Handler<AddFilewatcher> for SessionManager {
    type Result = ();

    fn handle(&mut self, msg: AddFilewatcher, _: &mut Context<Self>) -> Self::Result {
        self._filewatcher_addr = Some(msg.addr);
    }
}
