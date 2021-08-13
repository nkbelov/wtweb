use std::path::{Path, PathBuf};
use serde::{Deserialize};
use toml;
use std::fs;


pub struct Blog<'d> {
	source_dir: &'d Path,
	render_dir: &'d Path
}

#[derive(Deserialize)]
struct BlogConfig {
	posts: Vec<String>
}

impl<'d> Blog<'d> {

	pub fn new(source_dir: &'d Path, render_dir: &'d Path) -> Option<Self> {



		Some(Blog { source_dir, render_dir })
	}

}