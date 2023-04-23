use std::{path::PathBuf, str::FromStr};

use pulldown_cmark::{Parser, html::push_html};
use serde::{Serialize, Deserialize};
use handlebars::Handlebars;

/// Each variant directly corresponds to a partial template,
/// and each field in the variant corresponds to a
/// variable the partial accesses.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Content {
    Index,
    Article { text: String }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    content: Content,
    header: bool,
    pub boring: bool
}

impl Page {

    pub fn new(content: Content, boring: bool) -> Self {
        // Header should be visible for every page except index
        let header = !(matches!(content, Content::Index));

        Self {
            content,
            header,
            boring
        }
    }

}

pub fn render_markdown(md: &str) -> String {
    let md_parser = Parser::new(md);
    let mut result = String::new();
    push_html(&mut result, md_parser);
    result
}

pub fn render(page: &Page) -> String {
    let mut hb = Handlebars::new();
    let tpl_path = PathBuf::from_str("./templates").unwrap();
    hb.register_templates_directory(".hbs", &tpl_path).unwrap();

    hb.render("base", &page).unwrap()
}