use pulldown_cmark::{Parser, html};

pub fn render_default(md: &str) -> String {
	let parser = Parser::new(md);
	let mut result = String::new();
	html::push_html(&mut result, parser);
	result
}