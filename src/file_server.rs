//use actix_web::{web::Bytes};
use std::path::{PathBuf};
use std::fs;
use log::*;
use toml;
use serde::{Serialize, Deserialize};
use serde_json;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PostMeta {
	title: String,
	maturity: String,
	filename: String,
	path: String,
	tags: Vec<String>
}

type Posts = Vec<PostMeta>;

#[derive(Deserialize)]
struct Garden {
	posts: Posts
}


#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content="c")]
pub enum PageType {
	Index,
	E404,
	Post(String)
}

#[derive(Serialize, Debug)]
struct Post {
	text: String
}

#[derive(Serialize)]
struct Ctx {
	page_type: PageType,
	posts: Posts,
	post: Option<Post>
}

impl Ctx {
	fn new(page_type: PageType) -> Self {
		let path = "resources/garden/Garden.toml";
		let toml = fs::read_to_string(&path).unwrap();
		let garden: Garden = toml::from_str(&toml).unwrap();
		let posts = garden.posts;
		Self { page_type, posts, post: None }
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

	fn misc_path(&self, path: &str) -> PathBuf {
		let mut mpath = self.base_dir.clone();
		mpath.push("misc");
		mpath.push(path);
		mpath
	}

	fn mime(ext: &str) -> Option<&'static str> {
		match ext {
			"webmanifest" => Some("application/manifest+json"),
			"png" => Some("image/png"),
			"svg" => Some("image/svg+xml"),
			"ico" => Some("image/vnd.microsoft.icon"),
			"xml" => Some("application/xml"),
			_ => None
		}
	}


	fn read_post(&self, name: &str) -> Option<String> {
		let mut path = self.base_dir.clone();
		path.push("garden");
		path.push(name);
		path.set_extension("md");

		info!("Garden: Looking for {:?}", &path);

		fs::read_to_string(&path).ok()
	}

	pub async fn get_html(&self, t: PageType) -> Option<String> {
		// Prepare Handlebars
		let mut hbs = Handlebars::new();
		debug!("Getting page with {:?}", t);

		for tmpl in &vec!["base", "index", "post", "garden_preview", "p404"] {
			debug!("Registring template {}: {:?}", tmpl, hbs.register_template_file(tmpl, self.templ_path(tmpl)));
		}

		hbs.register_helper("markdown", Box::new(render_markdown));

		// Create and populate rendering context
		let mut ctx = Ctx::new(t.clone());
		if let PageType::Post(name) = &t {
			let post = self.read_post(name).and_then(|text| Some(Post { text }));
			debug!("Found {:?} for post with name {}", post, name);
			ctx.post = post;
		}
		debug!("Produced context: {:?}", serde_json::to_string_pretty(&ctx));

		let result = hbs.render("base", &ctx).ok()?;
		debug!("Successfully rendered HTML {:?}", &t);
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
		path.push("styles");
		path.set_extension("css");

		fs::read_to_string(&path).ok()
	}

	pub async fn get_index(&self) -> Option<String> {
		self.get_html(PageType::Index).await
	}

	pub async fn get_404(&self) -> Option<String> {
		self.get_html(PageType::E404).await
	}

	pub async fn get_post(&self, name: &str) -> Option<String> {
		self.get_html(PageType::Post(name.to_owned())).await
	}

	pub async fn get_misc(&self, path: &str) -> Option<(Vec<u8>, &'static str)> {
		let mpath = self.misc_path(path);
		let ext = mpath.extension()?.to_str()?;
		let mime = Self::mime(&ext)?;
		let bytes = fs::read(&mpath).ok()?;

		Some((bytes, mime))
	}

}