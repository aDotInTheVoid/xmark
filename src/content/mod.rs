// SPDX-License-Identifier: GPL-3.0-only
use ramhorns::Content as Rhc;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

pub(crate) mod collect;
pub(crate) mod pagetoc;
pub(crate) mod render;
pub(crate) use collect::Dirs;

/// The content in a suitable form.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Content(pub(crate) Vec<Book>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct Book {
    pub(crate) title: String,
    pub(crate) pages: Vec<Page>,
    /// List of files to be written, and the url to redirect to.
    pub(crate) redirects: Vec<(PathBuf, String)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Page {
    pub(crate) name: String,
    /// The html file to render to
    pub(crate) output: PathBuf,
    /// The md input file.
    pub(crate) input: PathBuf,
    /// The link to the next page
    pub(crate) next: Option<String>,
    /// The link to the previous
    pub(crate) prev: Option<String>,
    /// The pages parents, and their parents, and so on.
    ///
    /// Inspired by [github's docs](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/set-up-git)
    /// Where is has the heirachy "[GitHub.com](https://docs.github.com/en/free-pro-team@latest/github) /
    /// [Getting started](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github) /
    /// [Quickstart](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/quickstart) /
    /// [Set up Git](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/set-up-git)
    pub(crate) heirachy: Vec<Link>,
}

//TODO: Should this be the same as pagetoc::Link.
// This is relative to site root, so needs special care when we're serving
// on a subdir. that is just relative to the page
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Rhc)]
pub(crate) struct Link {
    pub(crate) prity: String,
    pub(crate) link: String,
}

#[cfg(test)]
mod tests {
    // TODO: these should be in the module that defined them

    use assert_fs::prelude::*;
    use cli::Args;
    use insta::{assert_yaml_snapshot, dynamic_redaction};

    use crate::cli::{self, config};

    use super::collect::output_loc;
    use super::*;

    fn test_output_loc(md: &str, out: &str, base: &str, expected: &str) {
        assert_eq!(
            output_loc(md.as_ref(), out.as_ref(), base.as_ref())
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
            expected
        );
    }

    #[test]
    fn output_locs() {
        test_output_loc(
            "/tmp/x/y.md",
            "/tmp/x/out",
            "/tmp/x/",
            "/tmp/x/out/y/index.html",
        );
        test_output_loc(
            "/tmp/x/z/README.md",
            "/tmp/x/out",
            "/tmp/x",
            "/tmp/x/out/z/index.html",
        );
        test_output_loc(
            "/tmp/x/z.md",
            "/tmp/x/out",
            "/tmp/x",
            "/tmp/x/out/z/index.html",
        );
        test_output_loc(
            "/tmp/zz/foo.md",
            "/xmark/",
            "/tmp/zz/",
            "/xmark/foo/index.html",
        );
    }

    fn test_page_url(out_file: &str, base_url: &str, out_dir: &str, expected: &str) {
        let page = Page {
            output: PathBuf::from(out_file),
            ..Default::default()
        };
        let dirs = Dirs {
            base_url: base_url.to_owned(),
            out_dir: PathBuf::from(out_dir),
            ..Default::default()
        };
        let url = page.url(&dirs).unwrap();
        assert_eq!(url, expected);
    }

    #[test]
    fn urls() {
        test_page_url("/out/x/y/index.html", "/", "/out", "/x/y");
        test_page_url(
            "/usr/src/fx/_out/html/book3/cd/f/index.html",
            "/books/",
            "/usr/src/fx/_out/html",
            "/books/book3/cd/f",
        )
    }

    #[test]
    fn dummy_e2e() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dummy-book"),
            &["xmark.toml", "book-*/**"],
        )
        .unwrap();

        let args = Args {
            dir: temp.path().to_owned(),
            ..Default::default()
        };
        let conf = config::load(&args).unwrap();
        let dirs = Dirs::new(&conf, &args);
        let content = Content::new(&conf, &dirs).unwrap();
        let tp = temp.path().as_os_str().to_str().unwrap().to_owned();

        // This is a amazing lifetime hack, because dynamic_redaction requires
        // it's closures to be 'static, and I can't think of a better way to get
        // arround this. Don't bother fixing unless you realy want fun lifetime
        // issues.
        let redaction = |string, db| {
            move |mut val, _: insta::internals::ContentPath| {
                // TODO: Dont do insta crimes.
                // This is in #[doc(Hidden)] internals.
                while let insta::internals::Content::Some(some) = val {
                    val = *some;
                }
                if let insta::internals::Content::String(s) = val {
                    if db {
                        dbg!(&s);
                    }
                    s.replace(&string, "BASEDIR").into()
                } else {
                    val
                }
            }
        };
        dbg!(&tp);
        assert_yaml_snapshot!(content,

            {
                 ".**.input" => dynamic_redaction(redaction(tp.clone(), false)),
                 ".*.redirects[][]" => dynamic_redaction(redaction(tp.clone(), true)),
                 ".**.output" => dynamic_redaction(redaction(tp, false)),
            }
        );
    }

    #[test]
    fn empty_conf() {
        let args = Default::default();
        let conf = Default::default();
        let content = Content::new(&conf, &args).unwrap();
        assert_eq!(content.0.len(), 0);
    }

    #[test]
    fn empty_book() {
        let args = Default::default();
        let book = Default::default();
        let conf = config::GlobalConf {
            books: vec![book],
            ..Default::default()
        };
        let content = Content::new(&conf, &args).unwrap();
        assert_eq!(content, Content(vec![Default::default()]));
    }
}
