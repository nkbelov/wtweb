use std::{path::PathBuf, str::FromStr};

use serde::{Serialize, Deserialize};
use handlebars::Handlebars;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "camelCase")]
pub enum Page {
    Index { name: String }
}

pub fn render(page: &Page) -> String {
    let mut hb = Handlebars::new();
    let tpl_path = PathBuf::from_str("./templates").unwrap();
    hb.register_templates_directory(".hbs", &tpl_path).unwrap();
    hb.render("base", &page).unwrap()
}