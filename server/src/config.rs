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
}

/// Config file after processing raw config
pub struct Config {
    /// Server port
    pub port: u16,
    /// Certificate
    pub certificate: Vec<u8>,
    /// Private key
    pub private_key: Vec<u8>,
}

/// Read config at ./config.toml
///
/// Will panic if file not exist OR file syntax wrong
pub fn get_config() -> Config {
    let RawConfig {
        port,
        pem_file_path,
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
    }
}
