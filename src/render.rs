use std::{path::PathBuf, str::FromStr};

use pulldown_cmark::Parser;
use serde::{Serialize, Deserialize};
use handlebars::Handlebars;

use crate::bookmark::push_html;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub text: Option<String>
}

impl Content {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum PageType {
    Index
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub r#type: PageType,
    pub content: Content
}

pub fn render(page: &Page) -> String {
    let mut hb = Handlebars::new();
    let tpl_path = PathBuf::from_str("./templates").unwrap();
    hb.register_templates_directory(".hbs", &tpl_path).unwrap();

    let mut page = page.clone();
    if let Some(md) = page.content.text.clone() {
        let md_parser = Parser::new(&md);
        let mut result = String::new();
        push_html(&mut result, md_parser);
        page.content.text = Some(result);
    }

    hb.render("base", &page).unwrap()
}