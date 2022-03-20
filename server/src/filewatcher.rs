use crate::messages::*;
use actix::prelude::*;
use log::{error, info};
use notify::DebouncedEvent;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

pub struct FileWatcherActor {
    manager_address: Addr<crate::manager::SessionManager>,
    rx: Receiver<DebouncedEvent>,
}

impl FileWatcherActor {
    pub fn new(
        manager_address: Addr<crate::manager::SessionManager>,
        rx: Receiver<DebouncedEvent>,
    ) -> Self {
        Self {
            manager_address,
            rx,
        }
    }
}

impl Actor for FileWatcherActor {
    type Context = SyncContext<Self>;
}

impl Handler<StartWatchingFiles> for FileWatcherActor {
    type Result = ();
    fn handle(&mut self, _msg: StartWatchingFiles, _ctx: &mut Self::Context) -> Self::Result {
        let mut changed_paths = vec![];
        while let Ok(debounced_event) = self.rx.recv() {
            retrieve_changed_paths_and_append_to_vec(debounced_event, &mut changed_paths);
            while let Ok(debounced_event) = self.rx.try_recv() {
                retrieve_changed_paths_and_append_to_vec(debounced_event, &mut changed_paths);
            }
            self.manager_address.do_send(FilePathHasSomethingChanged {
                paths: changed_paths,
            });
            changed_paths = vec![];
        }
    }
}

/// Use this pattern since there may be more than one retrieved path for one debounced event
fn retrieve_changed_paths_and_append_to_vec(
    debounced_event: DebouncedEvent,
    changed_paths: &mut Vec<PathBuf>,
) {
    use DebouncedEvent::*;
    match debounced_event {
        Create(path) => {
            info!("Detect file created at {:?}", path);
            changed_paths.push(path);
        }
        Write(path) => {
            info!("Detect file written at {:?}", path);
            changed_paths.push(path);
        }
        Remove(path) => {
            info!("Detect file removed at {:?}", path);
            changed_paths.push(path);
        }
        Rename(from, to) => {
            info!("Detect file renamed from {:?} to {:?}", from, to);
            changed_paths.push(from);
            changed_paths.push(to);
        }
        Error(error, path) => {
            error!(
                "Error detected in file watcher at path {:?}: {}",
                path, error
            );
        }
        _ => {}
    }
}
