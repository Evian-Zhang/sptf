use rustls_pemfile::Item;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::iter;

/// Raw Config in file format
#[derive(Deserialize)]
struct RawConfig {
    /// Server port
    port: u16,
    /// PEM file path
    pem_file_path: String,
    database_port: u16,
    database_username: String,
    database_password: String,
    redis_port: u16,
    redis_username: String,
    redis_password: String,
}

/// Config file after processing raw config
pub struct Config {
    /// Server port
    pub port: u16,
    /// Certificate
    pub certificate: Vec<u8>,
    /// Private key
    pub private_key: Vec<u8>,
    pub database_port: u16,
    pub database_username: String,
    pub database_password: String,
    pub redis_port: u16,
    pub redis_username: String,
    pub redis_password: String,
}

/// Read config at ./config.toml
///
/// Will panic if file not exist OR file syntax wrong
pub fn get_config() -> Config {
    let RawConfig {
        port,
        pem_file_path,
        database_port,
        database_username,
        database_password,
        redis_port,
        redis_username,
        redis_password,
    } = toml::from_str::<RawConfig>(&fs::read_to_string("./config.toml").unwrap()).unwrap();

    let mut certificate = None;
    let mut private_key = None;
    let mut reader = BufReader::new(File::open(pem_file_path).unwrap());
    for item in iter::from_fn(|| rustls_pemfile::read_one(&mut reader).transpose()) {
        match item.unwrap() {
            Item::X509Certificate(cert) => certificate = Some(cert),
            Item::RSAKey(rsa_key) => private_key = Some(rsa_key),
            Item::PKCS8Key(pcks8_key) => private_key = Some(pcks8_key),
            Item::ECKey(ec_key) => private_key = Some(ec_key),
            _ => {}
        }
    }
    let certificate = certificate.expect("No certificate detected.");
    let private_key = private_key.expect("No key detected.");

    Config {
        port,
        certificate,
        private_key,
        database_port,
        database_username,
        database_password,
        redis_port,
        redis_username,
        redis_password,
    }
}
