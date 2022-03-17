// extern crate protobuf;

mod config;
mod manager;
mod messages;
mod protos;
mod session;

use actix::prelude::*;
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use env_logger::Env;
use manager::SessionManager;
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use session::UserSession;

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    manager_address: web::Data<Addr<SessionManager>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        UserSession::new(manager_address.get_ref().clone()),
        &req,
        stream,
    );
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::get_config();
    let rustls_server_config = RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            vec![Certificate(config.certificate)],
            PrivateKey(config.private_key),
        )
        .unwrap();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let manager_address = SessionManager::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(manager_address.clone()))
            .route("/ws/", web::get().to(index))
            .wrap(Logger::default())
    })
    .bind_rustls(("127.0.0.1", config.port), rustls_server_config)?
    .run()
    .await
}
