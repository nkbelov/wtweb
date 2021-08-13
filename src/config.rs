use std::fs;
use std::fs::{File};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::io::BufReader;
use std::str::FromStr;
use std::path::PathBuf;

use toml;
use toml::Value;

use serde::{Deserialize};

use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};

#[derive(Debug, Deserialize)]
pub struct Config {
	host: Ipv4Addr,
	port: u16,
	use_tls: bool,
	temp_dir: String,

	tls_files: Option<TlsFiles>
}

#[derive(Debug, Deserialize)]
struct TlsFiles {
	certs: PathBuf,
	key: PathBuf
}

impl Config {

	pub fn load() -> Option<Self> {
		let bytes = fs::read("Config.toml").ok()?;
		let conf: Self = toml::from_slice(&bytes).ok()?;

		Some(conf)
	}

	pub fn temp_dir(&self) -> &str {
		&self.temp_dir
	}

	pub fn socket(&self) -> SocketAddrV4 {
		SocketAddrV4::new(self.host, self.port)
	}

	pub fn rustls_config(&self) -> Option<ServerConfig> {

		if !self.use_tls {
			return None;
		}

		let tls_files = match &self.tls_files {
			Some(files) => files,
			None => return None
		};

		
		let cert_file = &mut BufReader::new(File::open(&tls_files.certs).ok()?);
		let key_file = &mut BufReader::new(File::open(&tls_files.key).ok()?);
		let cert_chain = certs(cert_file).ok()?;
		let mut keys = pkcs8_private_keys(key_file).ok()?;

		let mut config = ServerConfig::new(NoClientAuth::new());
		// keys.remove(0) to take an owned value
		config.set_single_cert(cert_chain, keys.remove(0)).ok()?;
		return Some(config);
	}

}