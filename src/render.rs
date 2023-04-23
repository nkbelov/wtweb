use std::{path::PathBuf, str::FromStr, string::ParseError};

use pulldown_cmark::{Parser, html::push_html, Event, Tag, HeadingLevel, escape::escape_html};
use serde::Serialize;
use handlebars::Handlebars;

use crate::posts::Post;

/// Each variant directly corresponds to a partial template,
/// and each field in the variant corresponds to a
/// variable the partial accesses.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Content {
    Index { posts: Vec<Post> },
    Article { text: String }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    content: Content,
    header: bool,
    pub boring: bool
}

impl Page {

    pub fn new(content: Content, boring: bool) -> Self {
        // Header should be visible for every page except index
        let header = !(matches!(content, Content::Index {..}));

        Self {
            content,
            header,
            boring
        }
    }

}

pub fn extract_meta(md: &str) -> (String, Option<String>) {
    let parser = Parser::new(md);
    let mut title = String::new();
    let mut abs: Option<String> = None;

    let mut in_title = false;
    let mut in_abs = false;
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading(HeadingLevel::H1, _, _) => {
                        if title.is_empty() {
                            // Only open title if no h1 has been already encountered
                            in_title = true;
                        }
                    }
                    Tag::Abstract => {
                        if abs.is_none() {
                            abs = Some(String::new());
                            in_abs = true;
                        }
                    }
                    _ => {}
                }
                
            }
            Event::End(tag) => {
                match tag {
                    Tag::Heading(HeadingLevel::H1, _, _) => {
                        in_title = false;
                    }
                    Tag::Abstract => {
                        in_abs = false;
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_title {
                    escape_html(&mut title, &text).unwrap();
                } else if in_abs {
                    escape_html(abs.as_mut().unwrap(), &text).unwrap();
                }
            }
            _ => {}
        }
    }

    (title, abs)
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