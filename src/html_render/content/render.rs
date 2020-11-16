use std::fs;

use eyre::Result;
use pulldown_cmark::{html, Options, Parser};
use ramhorns::Content as Rhc;
use serde::Serialize;

use super::{Link, Page as CPage};

// Because we borrow link, we cant Deserialize, so snapshot tests may not work.
// If so, we can remove the Serialize bound
#[derive(Debug, Clone, Serialize, PartialEq, Rhc)]
pub struct Page<'a> {
    title: &'a str,
    inner_html: String,
    pub heirachy: &'a [Link],
    pub pagetoc: String,
    /// The link to the next page
    pub next: Option<&'a str>,
    /// The link to the previous
    pub prev: Option<&'a str>,
}

impl<'a> Page<'a> {
    pub fn new(from: &'a CPage) -> Result<Self> {
        // TODO: Don't buffer the whole input
        let inner_html = render_markdown(&fs::read_to_string(&from.input)?);

        Ok(Self {
            title: &from.name,
            inner_html,
            // TODO: The way to do this is to gen the pagetoc in render_markdown.
            pagetoc: String::new(),
            next: from.next.as_deref(),
            prev: from.prev.as_deref(),
            heirachy: &from.heirachy,
        })
    }
}

// TODO: A million customizations
pub(crate) fn render_markdown(content: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(content, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}
