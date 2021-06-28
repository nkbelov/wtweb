#[macro_use] extern crate rocket;

use toml;
use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::BTreeMap;
use rocket::response::content;
use rocket::State;
use handlebars::{Handlebars};


#[derive(Serialize, Deserialize)]
struct Strings {
	strings: BTreeMap<String, String>,
	arrays: BTreeMap<String, Vec<String>>
}

#[derive(Serialize, Deserialize)]
struct Config {
	sources: BTreeMap<String, String>,
	templates: BTreeMap<String, String>
}

type PageMap = BTreeMap<String, String>;

struct Pages<'a> {
	strings: Strings,
	hb: Handlebars<'a>,
	map: PageMap
}

impl Pages<'_> {
	fn new() -> Self {
		Pages { 
			strings: toml::from_str(&get_string("resources/strings.toml")).unwrap(),
			hb: Handlebars::new(),
		 	map: PageMap::new() 
		}
	}

	fn populate(&mut self, templs: &BTreeMap<String, String>) {
		dbg!(&self.strings.arrays);
		for (name, templ) in templs {
			let hb_source = get_string(templ);
			self.hb.register_template_string(name, hb_source);
			let page = self.hb.render(name, &self.strings).unwrap();
			self.map.insert(name.to_string(), page.clone());
		}
	}

	fn get(&self, name: &str) -> String {
		self.map.get(name).unwrap().to_string()
	}
}


fn get_string(path: &str) -> String {
	return fs::read_to_string(path).unwrap();
}

#[get("/")]
fn index(pages: &State<Pages>) -> content::Html<String> {
	let s = pages.get("index");
	return content::Html(s);
}

#[get("/styles", format = "css")]
fn styles() -> content::Css<String> {
	return content::Css(get_string("resources/styles.css"));
}

#[launch]
fn rocket() -> _ {

	let config: Config = toml::from_str(&get_string("resources/config.toml")).unwrap();

	let mut pages = Pages::new();
	pages.populate(&config.templates);

    rocket::build()
    	.mount("/", routes![index, styles])
    	.manage(pages)
}