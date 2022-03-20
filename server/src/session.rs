use crate::error::{ProtobufError, SPTFError};
use crate::messages::*;
use crate::protos::sptf::{BasicIncomingMessage, BasicOutcomingMessage};
use actix::prelude::*;
use actix_web_actors::ws;
use log::{info, warn};
use protobuf::Message;
use std::path::{Path, PathBuf};
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
    watched_path: Option<PathBuf>,
}

impl UserSession {
    /// Create a new user session
    pub fn new(manager_address: Addr<crate::manager::SessionManager>, user_id: Uuid) -> Self {
        Self {
            session_id: None,
            user_id,
            heartbeat: Instant::now(),
            manager_address,
            watched_path: None,
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
            Ok(ws::Message::Binary(bin)) => {
                let mut response = BasicOutcomingMessage::default();
                response.set_version(crate::common::PROTOCOL_VERSION);
                let request = match BasicIncomingMessage::parse_from_carllerche_bytes(&bin) {
                    Ok(request) => request,
                    Err(err) => {
                        warn!("Error parsing income message: {}", err);
                        let error = ProtobufError::WrongFormat;
                        response.set_GeneralError(error.to_proto_error());
                        ctx.binary(response.write_to_bytes().unwrap_or_else(|err| {
                            warn!("Failed to write to bytes: {}", err);
                            vec![]
                        }));
                        return;
                    }
                };
                let message_content = if let Some(message_content) = request.message_content {
                    message_content
                } else {
                    warn!("Incoming message has none message_content field");
                    let error = ProtobufError::WrongFormat;
                    response.set_GeneralError(error.to_proto_error());
                    ctx.binary(response.write_to_bytes().unwrap_or_else(|err| {
                        warn!("Failed to write to bytes: {}", err);
                        vec![]
                    }));
                    return;
                };
                use crate::protos::sptf::BasicIncomingMessage_oneof_message_content::*;
                match message_content {
                    ListDirectoryMessage(list_directory_request) => {
                        let list_directory_response =
                            crate::files::list_dir(&Path::new(list_directory_request.get_path()));
                        response.set_ListDirectoryResponse(list_directory_response);
                        ctx.binary(response.write_to_bytes().unwrap_or_else(|err| {
                            warn!("Failed to write to bytes: {}", err);
                            vec![]
                        }));
                    }
                }
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
