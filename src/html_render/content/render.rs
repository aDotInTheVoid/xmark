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
    #[ramhorns(flatten)]
    pub global: Global<'a>,
}

impl<'a> Page<'a> {
    pub fn new(from: &'a CPage, dirs: &'a super::Dirs) -> Result<Self> {
        // TODO: Don't buffer the whole input
        let inner_html = render_markdown(&fs::read_to_string(&from.input)?);

        let global = Global {
            path_to_root: &dirs.base_url,
            ..Default::default()
        };

        Ok(Self {
            title: &from.name,
            inner_html,
            // TODO: The way to do this is to gen the pagetoc in render_markdown.
            pagetoc: String::new(),
            next: from.next.as_deref(),
            prev: from.prev.as_deref(),
            heirachy: &from.heirachy,
            global,
        })
    }
}

/// Options every page needs not specific to a page
#[derive(Debug, Clone, Serialize, PartialEq, Rhc)]
pub struct Global<'a> {
    pub path_to_root: &'a str,
    pub language: &'a str,
    pub preferred_dark_theme: &'a str,
    pub default_theme: &'a str
}

impl<'a> Default for Global<'a> {
    fn default() -> Self {
        Global {
            path_to_root: "/",
            language: "en",
            // THESE ARE FACTS.
            default_theme: "rust",
            preferred_dark_theme: "coal"
        }
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
