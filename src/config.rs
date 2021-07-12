use std::fs;
use std::fs::{File};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::io::BufReader;
use std::str::FromStr;

use toml;
use toml::Value;


use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};


enum Field {
	Host,
	Port
}

impl Field {
	fn key(&self) -> &'static str {
		return match self {
			Field::Host => "host",
			Field::Port => "port"
		};
	}
}

pub struct Config {
	pub socket: SocketAddrV4,

	key: Option<String>,
	certs: Option<String>
}

pub struct TlsConfig<'c> {
	key: &'c str,
	certs: &'c str
}

type TomlMap = toml::map::Map<String, toml::Value>;

fn socket(map: &TomlMap) -> Option<SocketAddrV4> {
	let hostval = map.get(Field::Host.key())?;
	let portval = map.get(Field::Port.key())?;
	if let (Value::String(string), Value::Integer(int)) = (hostval, portval) {
		let host = Ipv4Addr::from_str(&string).ok()?;
		return Some(SocketAddrV4::new(host, *int as u16));
	}

	None
}

impl Config {

	pub fn load() -> Option<Self> {
		let bytes = fs::read("Config.toml").ok()?;
		let map: TomlMap = toml::from_slice(&bytes).ok()?;

		if let Some(socket) = socket(&map) {
			return Some(Config { socket, key: None, certs: None });
		}
		
		None
	}

	pub fn tls<'c>(&'c self) -> Option<TlsConfig<'c>> {
		if let (Some(key), Some(certs)) = (&self.key, &self.certs)  {
			let config = TlsConfig {
				key: key.as_ref(),
				certs: certs.as_ref()
			};

			return Some(config);
		}  else {
			return None;
		}
	}

	pub fn rustls_config(&self) -> Option<ServerConfig> {

		let tls_files = self.tls()?;
		let mut config = ServerConfig::new(NoClientAuth::new());
    	let cert_file = &mut BufReader::new(File::open(tls_files.certs).ok()?);
    	let key_file = &mut BufReader::new(File::open(tls_files.key).ok()?);
    	let cert_chain = certs(cert_file).ok()?;
    	let mut keys = pkcs8_private_keys(key_file).ok()?;
    	// keys.remove(0) to take an owned value
    	config.set_single_cert(cert_chain, keys.remove(0)).ok()?;
    	return Some(config);
	}

}