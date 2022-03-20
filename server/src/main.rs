// extern crate protobuf;

mod common;
mod config;
mod error;
mod files;
mod filewatcher;
mod manager;
mod messages;
mod protos;
mod session;
mod user;

use actix::prelude::*;
use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::ContentType,
    middleware::Logger,
    post,
    web::{self, Json, PayloadConfig},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use deadpool_postgres::{
    Manager as DeadpoolPostgresManager, ManagerConfig as DeadpoolPostgresManagerConfig, Pool,
};
use deadpool_redis::{Config as DeadpoolRedisConfig, Runtime as DeadpoolRedisRuntime};
use env_logger::Env;
use error::{FileError, SPTFError, UnexpectedError};
use filewatcher::FileWatcherActor;
use log::error;
use manager::SessionManager;
use notify::{RecursiveMode, Watcher};
use protobuf::Message;
use protos::sptf::FileUploadRequest;
use redis::{
    ConnectionAddr as RedisConnectionAddr, ConnectionInfo as RedisConnectionTotalInfo,
    RedisConnectionInfo,
};
use rustls::ServerConfig as RustlsServerConfig;
use serde::{Deserialize, Serialize};
use session::UserSession;
use std::sync::mpsc;
use tokio_postgres::{Config as PostgresConfig, NoTls};

/// Shared app data
struct AppData {
    /// Address of session manager
    manager_address: actix::Addr<manager::SessionManager>,
    /// Database connection pool
    database_connection_pool: deadpool::managed::Pool<deadpool_postgres::Manager>,
    /// Redis connection pool
    redis_connection_pool:
        deadpool::managed::Pool<deadpool_redis::Manager, deadpool_redis::Connection>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    uuid: String,
}

#[post("/login")]
async fn login(login_request: Json<LoginRequest>, app_data: web::Data<AppData>) -> HttpResponse {
    let postgres_client_fut = async {
        app_data
            .database_connection_pool
            .get()
            .await
            .map_err(|err| {
                error!(
                    "Failed to get a connection from postgres connection pool: {}",
                    err
                );
                UnexpectedError.to_boxed_self()
            })
    };
    let redis_connection_fut = async {
        app_data.redis_connection_pool.get().await.map_err(|err| {
            error!(
                "Failed to get a connection from redis connection pool: {}",
                err
            );
            UnexpectedError.to_boxed_self()
        })
    };
    let validate_result = user::validate_user(
        postgres_client_fut,
        redis_connection_fut,
        &login_request.username,
        &login_request.password,
    )
    .await;
    let uuid = match validate_result {
        Ok(uuid) => uuid,
        Err(error) => {
            return error.to_http_response();
        }
    };

    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::to_string(&LoginResponse {
            uuid: uuid.to_string(),
        })
        .unwrap(),
    )
}

#[derive(Deserialize)]
struct DownloadFilesQuery {
    paths: Vec<String>,
}

#[get("/download")]
async fn download_files(req: HttpRequest, query: web::Query<DownloadFilesQuery>) -> HttpResponse {
    match &query.paths[..] {
        [] => UnexpectedError.to_http_response(),
        [path] => match NamedFile::open(path) {
            Ok(named_file) => named_file.prefer_utf8(true).into_response(&req),
            Err(err) => {
                error!("Failed to open file {}: {}", path, err);
                FileError::PermissionDenied.to_http_response()
            }
        },
        _ => match files::compress_files(&query.paths).await {
            Ok(compressed_file) => match NamedFile::from_file(compressed_file, "target.tar.gz") {
                Ok(named_file) => named_file.prefer_utf8(true).into_response(&req),
                Err(err) => {
                    error!("Failed to open compressed file: {}", err);
                    FileError::PermissionDenied.to_http_response()
                }
            },
            Err(err) => err.to_http_response(),
        },
    }
}

#[post("/upload")]
async fn upload_files(body: web::Bytes) -> HttpResponse {
    let file_upload_request = match FileUploadRequest::parse_from_carllerche_bytes(&body) {
        Ok(file_upload_request) => file_upload_request,
        Err(err) => {
            error!("Failed to parse file upload request: {}", err);
            return UnexpectedError.to_http_response();
        }
    };
    if let Err(err) = files::upload_files(file_upload_request).await {
        err.to_http_response()
    } else {
        HttpResponse::Ok().finish()
    }
}

#[derive(Deserialize)]
struct WebsocketEstablishRequestQuery {
    auth_token: String,
}

#[get("/ws")]
async fn index(
    req: HttpRequest,
    stream: web::Payload,
    app_data: web::Data<AppData>,
    query: web::Query<WebsocketEstablishRequestQuery>,
) -> Result<HttpResponse, Error> {
    let auth_token_string = &query.auth_token;
    let redis_connection_fut = async {
        app_data.redis_connection_pool.get().await.map_err(|err| {
            error!(
                "Failed to get a connection from redis connection pool: {}",
                err
            );
            UnexpectedError.to_boxed_self()
        })
    };
    let auth_token_validate_result =
        user::validate_auth_token(redis_connection_fut, auth_token_string).await;
    let user_id = match auth_token_validate_result {
        Ok(user_id) => user_id,
        Err(error) => {
            return Ok(error.to_http_response());
        }
    };
    let resp = ws::start(
        UserSession::new(app_data.manager_address.clone(), user_id),
        &req,
        stream,
    );
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::get_config();

    // Config TLS support
    let rustls_server_config = RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(config.certificate_chain, config.private_key)
        .unwrap();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Config database
    let mut database_config = PostgresConfig::default();
    database_config
        .user(&config.database_username)
        .password(&config.database_password)
        .host("0.0.0.0")
        .port(config.database_port);
    let deadpool_postgres_manager_config = DeadpoolPostgresManagerConfig::default();
    let deadpool_postgres_manager = DeadpoolPostgresManager::from_config(
        database_config,
        NoTls,
        deadpool_postgres_manager_config,
    );
    let postgres_pool = Pool::builder(deadpool_postgres_manager)
        .max_size(16)
        .build()
        .unwrap();

    // Config session manager actor
    let manager_address = SessionManager::new().start();

    // Config filewatcher
    let cloned_manager_address = manager_address.clone();
    let filewatcher_addr = SyncArbiter::start(1, move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, common::FILEWATCHER_DEBOUNCE_DURATION)
            .expect("Unable to setup watcher");
        watcher
            .watch(&config.sptf_path, RecursiveMode::Recursive)
            .expect("Unable to setup watcher");
        FileWatcherActor::new(cloned_manager_address.clone(), rx)
    });
    filewatcher_addr.do_send(crate::messages::StartWatchingFiles);
    manager_address.do_send(crate::messages::AddFilewatcher {
        addr: filewatcher_addr,
    });

    // Config redis cache
    let redis_connection_total_info = RedisConnectionTotalInfo {
        addr: RedisConnectionAddr::Tcp("0.0.0.0".to_owned(), config.redis_port),
        redis: RedisConnectionInfo {
            username: Some(config.redis_username),
            password: Some(config.redis_password),
            ..RedisConnectionInfo::default()
        },
    };
    let deadpool_redis_config = DeadpoolRedisConfig {
        connection: Some(redis_connection_total_info.into()),
        ..DeadpoolRedisConfig::default()
    };
    let redis_pool = deadpool_redis_config
        .create_pool(Some(DeadpoolRedisRuntime::Tokio1))
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                manager_address: manager_address.clone(),
                database_connection_pool: postgres_pool.clone(),
                redis_connection_pool: redis_pool.clone(),
            }))
            .app_data(PayloadConfig::default().limit(common::MAX_FILE_UPLOAD_SIZE))
            .service(index)
            .service(download_files)
            .service(upload_files)
            .wrap(Logger::default())
    })
    .bind_rustls(("0.0.0.0", config.port), rustls_server_config)?
    .run()
    .await
}
