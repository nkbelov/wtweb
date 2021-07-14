use actix_web::{web::Bytes};
use std::path::{PathBuf};
use std::fs;
use log::info;
use toml;
use handlebars::{Handlebars};

type TomlMap = toml::map::Map<String, toml::Value>;

pub struct FileServer {
	base_dir: PathBuf
}


impl FileServer {

	pub fn in_dir(base_dir: PathBuf) -> Self {
		assert!(base_dir.is_dir());
		FileServer { base_dir }
	}

	fn construct_hbs(&self, name: &str) -> Option<String> {
		// Find a .hbs file with the base name, bail if there is none

		let mut path = self.base_dir.clone();
		path.push(name);
		path.set_extension("hbs");

		if let Ok(tmpl) = fs::read_to_string(&path) {
			// Found the template, let's find the strings
			path.set_extension("toml");
			if let Ok(strs) = fs::read_to_string(&path) {
				info!("Found template and strings for \"{}\"", name);

				let map: TomlMap = toml::from_str(&strs).unwrap();
				let hbs = Handlebars::new();
				//assert!(hbs.register_template_string(name, &tmpl).is_ok());
				return hbs.render_template(&tmpl, &map).ok();
			}
		}

		info!("Template and strings for \"{}\" not found", name);

		None
	}

	fn get_html(&self, name: &str) -> Option<String> {

		let mut path = self.base_dir.clone();
		path.push(name);
		path.set_extension("html");
		
		info!("Looking for {:?}", &path);

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

	pub async fn get_image(&self, name: &str) -> Option<Vec<u8>> {
		let mut path = self.base_dir.clone();
		path.push("images");
		path.push(name);

		info!("Looking for {:?}", &path);

		fs::read(&path).ok()
	}

	pub async fn get_styles(&self) -> Option<String> {
		let mut path = self.base_dir.clone();
		path.push("styles");
		path.set_extension("css");

		fs::read_to_string(&path).ok()
	}

	pub async fn get_index(&self) -> Option<String> {
		self.get_html("index")
	}

	pub async fn get_404(&self) -> Option<String> {
		self.get_html("404")
	}

}