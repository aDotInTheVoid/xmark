
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub mod collect;
pub mod pagetoc;
pub use collect::Dirs;

/// The content in a suitable form.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content(pub Vec<Book>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Book {
    pub title: String,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Page {
    pub name: String,
    /// The html file to render to
    pub output: PathBuf,
    /// The md input file.
    pub input: PathBuf,
    pub toc: pagetoc::PageToc,
    /// The link to the next page
    pub next: Option<String>,
    /// The link to the previous
    pub prev: Option<String>,
    /// The pages parents, and their parents, and so on.
    ///
    /// Inspired by [github's docs](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/set-up-git)
    /// Where is has the heirachy "[GitHub.com](https://docs.github.com/en/free-pro-team@latest/github) /
    /// [Getting started](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github) /
    /// [Quickstart](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/quickstart) /
    /// [Set up Git](https://docs.github.com/en/free-pro-team@latest/github/getting-started-with-github/set-up-git)
    pub heirachy: Vec<Link>,
}
// Oh dear god the allocations

/// Fun helper type
///
/// 1. Foo
/// 2. Bar
/// 2.1. Baz
/// 2.1.1 Quix
/// 2.2 Spam
///
/// Chapter(Foo)
/// Chapter(Bar)
/// StartSection
/// Chapter(Baz)
/// StartSection
enum PageListParts<'a> {
    //TODO: A better name
    Chapter(&'a ()),
    StartSection,
    EndSection,
}

//TODO: Should this be the same as pagetoc::Link.
// This is relative to site root, so needs special care when we're serving
// on a subdir. that is just relative to the page
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub prity: String,
    pub link: String,
}



pub fn output_loc(input_loc: &Path, out_dir: &Path, base_dir: &Path) -> Result<PathBuf> {
    let mut path = out_dir.join(input_loc.strip_prefix(base_dir)?);
    if path.file_name() == Some(OsStr::new("README.md")) {
        path.set_file_name("index.html")
    } else {
        path.set_extension("");
        path.push("index.html");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use insta::{assert_yaml_snapshot, dynamic_redaction};

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

        let args = cli::Args {
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
        let redaction = |string| {
            move |mut val, _: insta::internals::ContentPath| {
                // TODO: Dont do insta crimes.
                // This is in #[doc(Hidden)] internals.
                while let insta::internals::Content::Some(some) = val {
                    val = *some;
                }
                if let insta::internals::Content::String(s) = val {
                    s.replace(&string, "BASEDIR").into()
                } else {
                    val
                }
            }
        };

        assert_yaml_snapshot!(content,
            {
                 ".**.input" => dynamic_redaction(redaction(tp.clone())),
                 ".**.output" => dynamic_redaction(redaction(tp)),
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
        let conf = GlobalConf {
            books: vec![book],
            ..Default::default()
        };
        let content = Content::new(&conf, &args).unwrap();
        assert_eq!(content, Content(vec![Default::default()]));
    }
}
