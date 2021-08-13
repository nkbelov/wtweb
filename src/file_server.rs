//use actix_web::{web::Bytes};
use std::path::{PathBuf};
use std::fs;
use log::*;
use toml;
use serde::{Serialize, Deserialize};
//use serde_json;
use handlebars::*;
//use std::collections::HashMap;


use crate::markdown;

//type Json = serde_json::Value;

fn render_markdown(h: &Helper<'_, '_>, _: &Handlebars<'_>, _: &Context, _: &mut RenderContext<'_, '_>, out: &mut dyn Output) -> HelperResult {
	let md = h.param(0).and_then(|p| p.value().as_str()).ok_or(RenderError::new("Markdown string not found"))?;
	let result = markdown::render_default(&md);//.ok_or(RenderError::new("Unable to render markdown string"))?;
	out.write(&result)?;
	
	Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct GardenPrev {
	title: String,
	maturity: String
}

type Posts = Vec<GardenPrev>;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum PageType {
	Index,
	P404
}

#[derive(Serialize)]
struct Ctx {
	page_type: PageType,
	posts: Posts
}

#[derive(Deserialize)]
struct Garden {
	posts: Posts
}

impl Ctx {
	fn new(page_type: PageType) -> Self {
		let path = "resources/garden/Garden.toml";
		let toml = fs::read_to_string(&path).unwrap();
		let garden: Garden = toml::from_str(&toml).unwrap();
		let posts = garden.posts;
		dbg!(&posts);
		Self { page_type, posts }
	}
}

pub struct FileServer {
	base_dir: PathBuf
}


impl FileServer {

	pub fn in_dir(base_dir: PathBuf) -> Self {
		assert!(base_dir.is_dir());
		FileServer { base_dir }
	}

	fn templ_path(&self, name: &str) -> PathBuf {
		let mut path = self.base_dir.clone();
		path.push("templates");
		path.push(name);
		path.set_extension("hbs");
		path
	}

	fn page_type(name: &str) -> PageType {
		match name {
			"index" => PageType::Index,
			"p404" => PageType::P404,
			_ => PageType::P404
		}
	}

	pub async fn get_html(&self, name: &str) -> Option<String> {
		let mut hbs = Handlebars::new();
		debug!("Getting page with {}", name);

		for tmpl in &vec!["base", "index", "garden_post", "garden_preview", "p404"] {
			debug!("Registring template {}: {:?}", tmpl, hbs.register_template_file(tmpl, self.templ_path(tmpl)));
		}

		hbs.register_helper("markdown", Box::new(render_markdown));

		let ctx = Ctx::new(Self::page_type(name));

		let result = hbs.render("base", &ctx).ok()?;
		debug!("Successfully rendered HTML {}", name);
		Some(result)
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
		self.get_html("index").await
	}

	pub async fn get_404(&self) -> Option<String> {
		self.get_html("p404").await
	}

}