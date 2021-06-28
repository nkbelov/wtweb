#[macro_use] extern crate rocket;

use std::fs;
use rocket::response::content;

fn get_string(path: &str) -> String {
	return fs::read_to_string(path).unwrap();
}

#[get("/")]
fn index() -> content::Html<String> {
	let s = get_string("resources/index.html");
	return content::Html(s);
}

#[get("/styles", format = "css")]
fn styles() -> content::Css<String> {
	return content::Css(get_string("resources/styles.css"));
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    	.mount("/", routes![index, styles])
}