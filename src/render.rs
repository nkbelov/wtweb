use std::{path::PathBuf, str::FromStr};

use serde::{Serialize, Deserialize};
use handlebars::Handlebars;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Content {

}

impl Content {
    pub fn new() -> Self {
        Self {  }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PageType {
    Index
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub r#type: PageType,
    pub content: Content
}

pub fn render(page: &Page) -> String {
    let mut hb = Handlebars::new();
    let tpl_path = PathBuf::from_str("./templates").unwrap();
    hb.register_templates_directory(".hbs", &tpl_path).unwrap();
    hb.render("base", &page).unwrap()
}