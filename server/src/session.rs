use crate::messages::*;
use crate::protos::sptf::BasicIncomingMessage;
use actix::prelude::*;
use actix_web_actors::ws;
use log::info;
use protobuf::Message;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// User session actor
pub struct UserSession {
    /// Unique ID indicating self to session manager
    session_id: Option<usize>,
    /// Unique ID indicating self to user manager
    user_id: Uuid,
    /// Last heartbeat time
    ///
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    heartbeat: Instant,
    /// Address of session manager
    manager_address: Addr<crate::manager::SessionManager>,
    /// User watched paths
    watched_paths: Vec<PathBuf>,
}

impl UserSession {
    /// Create a new user session
    pub fn new(manager_address: Addr<crate::manager::SessionManager>, user_id: Uuid) -> Self {
        Self {
            session_id: None,
            user_id,
            heartbeat: Instant::now(),
            manager_address,
            watched_paths: vec![],
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn start_beating_heart(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                info!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for UserSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_beating_heart(ctx);

        let addr = ctx.address();
        self.manager_address
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(new_id) => act.session_id = Some(new_id),
                    // something is wrong with session manager
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if let Some(id) = self.session_id {
            // notify session manager
            self.manager_address.do_send(Disconnect { id });
        }
        Running::Stop
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                let request =
                    if let Ok(request) = BasicIncomingMessage::parse_from_carllerche_bytes(&bin) {
                        request
                    } else {
                        unimplemented!()
                    };
                let message_content = if let Some(message_content) = request.message_content {
                    message_content
                } else {
                    unimplemented!()
                };
                use crate::protos::sptf::BasicIncomingMessage_oneof_message_content::*;
                match message_content {
                    list_directory_message(list_directory_request) => {
                        // crate::files::list_dir()
                    }
                    download_files_message(download_files_request) => {
                        unimplemented!()
                    }
                    upload_files_message(upload_files_request) => {
                        unimplemented!()
                    }
                }
                unimplemented!()
            }
            _ => (),
        }
    }
}

impl Handler<RefreshFilesMessage> for UserSession {
    type Result = ();

    fn handle(
        &mut self,
        msg: RefreshFilesMessage,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Self::Result {
        unimplemented!()
    }
}
