#[macro_use] extern crate rocket;

use toml;
use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::BTreeMap;
use rocket::response::content;
use rocket::http::ContentType;
use rocket::State;
use handlebars::{Handlebars};


#[derive(Serialize, Deserialize)]
struct Strings {
	strings: BTreeMap<String, String>,
	arrays: BTreeMap<String, Vec<String>>
}

#[derive(Serialize, Deserialize)]
struct Config {
	rerender: bool,
	sources: BTreeMap<String, String>,
	templates: BTreeMap<String, String>
}

type PageMap = BTreeMap<String, String>;

struct Pages {
	templates: PageMap,
}

impl Pages {
	fn new(templates: PageMap) -> Self {
		Pages { 
		 	templates
		}
	}

	fn get(&self, name: &str) -> String {
		let strings: Strings = toml::from_str(&get_string("resources/strings.toml")).unwrap();
		let templ_path = self.templates.get(name).unwrap();
		let templ_string = get_string(&templ_path);
		let mut hb = Handlebars::new();
		hb.register_template_string(name, templ_string);
		hb.render(name, &strings).unwrap()
	}
}


fn get_string(path: &str) -> String {
	return fs::read_to_string(path).unwrap();
}

fn get_file(name: &str) -> Option<fs::File> {
	let path = std::path::Path::new("resources").join(&name);
	return fs::File::open(path).ok();
}

#[get("/")]
fn index(pages: &State<Pages>, config: &State<Config>) -> content::Html<String> {
	let s: String = pages.get("index");
	return content::Html(s);
}

#[get("/<filename>", format = "image/webp")]
fn image_png(filename: &str) -> Option<(ContentType, fs::File)> {
	if let Some(file) = get_file(filename) {
		return Some((ContentType::PNG, file));
	}
	
	Option::None
}

#[get("/styles", format = "css")]
fn styles() -> content::Css<String> {
	return content::Css(get_string("resources/styles.css"));
}

#[launch]
fn rocket() -> _ {

	let config: Config = toml::from_str(&get_string("resources/config.toml")).unwrap();

	let mut pages = Pages::new(config.templates.clone());

    rocket::build()
    	.mount("/", routes![index, styles, image_png])
    	.manage(pages)
    	.manage(config)
}