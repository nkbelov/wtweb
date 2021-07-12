use actix_web::{Responder};
use std::path::{PathBuf};
use std::fs;
use log::info;

pub struct FileServer {
	base_dir: PathBuf
}


impl FileServer {

	pub fn in_dir(base_dir: PathBuf) -> Self {
		FileServer { base_dir }
	}

	fn construct_hbs(&self, name: &str) -> Option<String> {
		Some("abc".to_string())
	}

	fn get_html(&self, name: &str) -> Option<String> {

		let mut path = self.base_dir.clone();
		path.set_file_name(name);
		path.set_extension(".html");

		if let Ok(str) = fs::read_to_string(&path) {
			info!("Serving existing HTML \"{}\"", name);
			return Some(str);
		}

		if let Some(str) = self.construct_hbs(name) {
			info!("Serving rendered HTML \"{}\"", name);
			return Some(str);
		}

		info!("Requested an HTML file \"{}\" but it does not exist", name);

		None
	}

	pub fn get_index(&self) -> impl Responder {
		self.get_html("index")
	}

	pub fn get_404(&self) -> Option<String> {
		self.get_html("404")
	}

}