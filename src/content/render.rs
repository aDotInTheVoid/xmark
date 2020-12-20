use std::cmp::Ordering;
use std::{fmt, fs};

use eyre::Result;
use pulldown_cmark::{html, Options, Parser};
use ramhorns::Content as Rhc;
use serde::Serialize;
use tracing::instrument;

use crate::html_render::HTMLRender;

use super::{Book, Link, Page as CPage};

// Because we borrow link, we cant Deserialize, so snapshot tests may not work.
// If so, we can remove the Serialize bound
#[derive(Debug, Clone, Serialize, PartialEq, Rhc)]
pub(crate) struct Page<'a> {
    title: &'a str,
    inner_html: String,
    pub(crate) heirachy: &'a [Link],
    pub(crate) pagetoc: String,
    /// The link to the next page
    pub(crate) next: Option<&'a str>,
    /// The link to the previous
    pub(crate) prev: Option<&'a str>,
    // This is unique to each chap, as the current page is highlighted.
    pub(crate) toc: String,
    #[ramhorns(flatten)]
    pub(crate) global: Global<'a>,
}

impl<'a> Page<'a> {
    #[instrument]
    pub(crate) fn new(from: &'a CPage, rd: &'a HTMLRender<'a>, book: &Book) -> Result<Self> {
        // TODO: Don't buffer the whole input
        let inner_html = render_markdown(&fs::read_to_string(&from.input)?);

        let global = Global {
            path_to_root: &rd.dirs.base_url,
            ..Default::default()
        };

        let mut toc = String::new();
        Self::write_toc(&mut toc, book, rd, from).unwrap();

        Ok(Self {
            title: &from.name,
            inner_html,
            // TODO: The way to do this is to gen the pagetoc in render_markdown.
            pagetoc: String::new(),
            next: from.next.as_deref(),
            prev: from.prev.as_deref(),
            heirachy: &from.heirachy,
            global,
            toc,
        })
    }

    // TODO: This doesn't have draft pages, which have been eliminated earlyer.
    // This requires rearchetecting the whole thing.
    // https://github.com/rust-lang/mdBook/blob/e5f74b6c8674bf23ed9c8d9b702fc9be7d409f1d/src/renderer/html_handlebars/helpers/toc.rs#L38-L146
    #[instrument]
    fn write_toc(out: &mut String, book: &Book, rd: &HTMLRender<'_>, this: &CPage) -> fmt::Result {
        //TODO: Alloc size
        out.push_str("<ol class=\"chapter\">");
        let mut current_level = 1;

        for i in &book.pages {
            // Every root page in a book also has the book it's in as a heirachy
            // element, which we dont care about.
            let level = i.heirachy.len() - 1;
            match level.cmp(&current_level) {
                Ordering::Greater => {
                    while level > current_level {
                        out.push_str("<li>");
                        out.push_str("<ol class=\"section\">");
                        current_level += 1;
                    }
                }
                Ordering::Less => {
                    while level < current_level {
                        out.push_str("</ol>");
                        out.push_str("</li>");
                        current_level -= 1;
                    }
                }
                Ordering::Equal => {}
            }
            out.push_str("<li class=\"chapter-item expanded\">");

            let mut href = rd.dirs.base_url.clone();
            href.push_str(
                i.output
                    .strip_prefix(&rd.dirs.out_dir)
                    .unwrap()
                    .to_str()
                    .unwrap(),
            );
            out.push_str("<a href=\"");
            out.push_str(&href);
            out.push('"');
            if this == i {
                out.push_str(" class=\"active\"")
            }
            out.push('>');
            out.push_str(&i.name);
            out.push_str("</a>");
            out.push_str("</li>");
        }

        while current_level > 1 {
            out.push_str("</ol>");
            out.push_str("</li>");
            current_level -= 1;
        }
        out.push_str("</ol>");

        Ok(())
    }
}

/// Options every page needs not specific to a page
#[derive(Debug, Clone, Serialize, PartialEq, Rhc)]
pub(crate) struct Global<'a> {
    pub(crate) path_to_root: &'a str,
    pub(crate) language: &'a str,
    pub(crate) preferred_dark_theme: &'a str,
    pub(crate) default_theme: &'a str,
}

impl<'a> Default for Global<'a> {
    fn default() -> Self {
        Global {
            path_to_root: "/",
            language: "en",
            // THESE ARE FACTS.
            default_theme: "rust",
            preferred_dark_theme: "coal",
        }
    }
}

// TODO: A million customizations
#[instrument]
pub(crate) fn render_markdown(content: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(content, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}
