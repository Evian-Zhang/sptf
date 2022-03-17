// extern crate protobuf;

mod config;
mod protos;
mod session;

use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use env_logger::Env;
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use session::UserSession;

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(UserSession::new(), &req, stream);
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

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/ws/", web::get().to(index))
    })
    .bind_rustls(("127.0.0.1", config.port), rustls_server_config)?
    .run()
    .await
}
