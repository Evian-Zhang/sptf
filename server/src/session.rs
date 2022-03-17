use crate::messages::*;
use actix::prelude::*;
use actix_web_actors::ws;
use log::info;
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// User session actor
pub struct UserSession {
    /// Unique ID indicating self to session manager
    id: Option<usize>,
    /// Last heartbeat time
    ///
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    heartbeat: Instant,
    /// Address of session manager
    manager_address: Addr<crate::manager::SessionManager>,
}

impl UserSession {
    /// Create a new user session
    pub fn new(manager_address: Addr<crate::manager::SessionManager>) -> Self {
        Self {
            id: None,
            heartbeat: Instant::now(),
            manager_address,
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

        // TODO: Add authentication here?
        let addr = ctx.address();
        self.manager_address
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(new_id) => act.id = Some(new_id),
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if let Some(id) = self.id {
            // notify chat server
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
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
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
