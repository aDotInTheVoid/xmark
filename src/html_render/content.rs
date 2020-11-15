use crate::config::{self, GlobalConf};
use crate::{cli, summary};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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
#[derive(Debug, Clone, Default)]
pub struct Dirs {
    out_dir: PathBuf,
    base_dir: PathBuf,
    base_url: String,
}

impl Dirs {
    pub fn new(conf: &GlobalConf, args: &cli::Args) -> Self {
        Self {
            base_dir: args.dir.clone(),
            out_dir: args.dir.join("_out").join("html"),
            base_url: conf
                .html
                .site_url
                .to_owned()
                .unwrap_or_else(|| "/".to_owned()),
        }
    }
}

impl Content {
    pub fn new(config: &config::GlobalConf, dirs: &Dirs) -> Result<Self> {
        Ok(Self(
            config
                .books
                .iter()
                .map(|x| Book::new(x, dirs))
                .collect::<Result<_>>()?,
        ))
    }
}

impl Book {
    pub fn new(book: &config::Book, dirs: &Dirs) -> Result<Self> {
        let title = book.summary.title.clone();
        let pages = Self::capture_pages(book, dirs)?;

        Ok(Self { title, pages })
    }

    fn capture_pages(book: &config::Book, dirs: &Dirs) -> Result<Vec<Page>> {
        use PageListParts::*;

        // We need to hold onto a bungh of stuff as we walk the tree, ands its
        // nicer if thats a list, and we just preserve the tree structure by
        // saying when we go up and down.

        // Create flat list
        let mut pages_parts = Vec::new();
        for i in &book.summary.prefix_chapters {
            pages_parts.push(Chapter(i));
        }
        for i in &book.summary.numbered_chapters {
            Self::capture_raw_parts(i, &mut pages_parts);
        }
        for i in &book.summary.suffix_chapters {
            pages_parts.push(Chapter(i));
        }

        let mut out = Vec::<Page>::new();
        let mut heirachy = vec![Link {
            prity: book.summary.title.clone(),
            link: Path::new(&dirs.base_url)
                .join(book.location.strip_prefix(&dirs.base_dir)?)
                .into_os_string()
                .into_string()
                .map_err(|x| eyre::eyre!("Invalid string {:?}", x))?,
        }];

        for i in pages_parts {
            match i {
                PageListParts::Chapter(chap) => {
                    let input = match &chap.location {
                        // Skip over the drafts, as they only show up in big toc.
                        None => continue,
                        Some(l) => l,
                    }
                    .clone();

                    let name = chap.name.clone();

                    // This is quite wastefull in terms of allocs, but who cares
                    let heirachy = heirachy.clone();

                    let output = output_loc(&input, &dirs.out_dir, &dirs.base_dir)?;
                    let mut page = Page {
                        input,
                        name,
                        output,
                        heirachy,
                        // TODO: I think this is the wrong design, as toc can't
                        // be determined utill we read the file, which we arn't
                        // doing here.
                        toc: Default::default(),
                        prev: None,
                        next: None,
                    };
                    page.heirachy.push(page.heirachy_element(dirs)?);
                    out.push(page)
                }
                PageListParts::StartSection => {
                    heirachy.push(out.last().unwrap().heirachy_element(dirs)?);
                }
                PageListParts::EndSection => {
                    heirachy.pop();
                }
            }
        }
        // We can't have something nice like
        // for i in out.windows_mut(2) {
        //     let before = &i[0];
        //     let after = &i[1];
        //     before.next = Some(after.url());
        //     after.prev = Some(before.url());
        // }
        // Because GAT's.
        for i in 0..out.len().saturating_sub(1) {
            let before_url = out[i].url(dirs)?;
            let after_url = out[i + 1].url(dirs)?;
            out[i].next = Some(after_url);
            out[i + 1].prev = Some(before_url);
        }

        Ok(out)
    }

    fn capture_raw_parts<'a>(link: &'a summary::Link, out: &mut Vec<PageListParts<'a>>) {
        use PageListParts::*;

        out.push(Chapter(&link.chapter));
        if !link.nested_items.is_empty() {
            out.push(StartSection);
            for i in &link.nested_items {
                Self::capture_raw_parts(i, out);
            }
            out.push(EndSection)
        }
    }
}

impl Page {
    pub fn heirachy_element(&self, dirs: &Dirs) -> Result<Link> {
        Ok(Link {
            prity: self.name.clone(),
            link: self.url(dirs)?,
        })
    }

    pub fn url(&self, dirs: &Dirs) -> Result<String> {
        let relative_pos = self.output.strip_prefix(&dirs.out_dir)?;
        let mut url = Path::new(&dirs.base_url).join(relative_pos);
        url.pop();
        Ok(
            url.into_os_string()
                .into_string()
                .map_err(|x| eyre::eyre!("Invalid string {:?}", x))?
                .replace("/./", "/"), // Hack
        )
    }
}
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
    Chapter(&'a summary::Chapter),
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

mod pagetoc {
    //! The "minitoc" for the page
    //!
    //! This is inspired by the right hand table of contents from
    //! [MkDocs Material](https://squidfunk.github.io/mkdocs-material/getting-started/)
    //!
    //! It's different from the left hand table for contents, which is for the whole book.
    //! This is just for the the page we're on
    //!
    //! A page with this content
    //! ```markdown
    //! # Controll Flow
    //!
    //! ## If
    //! ### If else
    //! ### As an expression
    //!
    //! ## Match
    //!
    //! ## Loops
    //! ### for .. in ..
    //! ### while ...
    //! ### loop
    //! ```
    //!
    //! Will have this as a toc
    //! ```rust,notest
    //! # https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=14a9939a1d620cd67a9c8763a730a6df
    //! PageToc(
    //!    [
    //!        H2 {
    //!            this: Link {
    //!                pritty: "If",
    //!                link: "if",
    //!            },
    //!            children: [
    //!                H3(
    //!                    Link {
    //!                        pritty: "If else",
    //!                        link: "if-else",
    //!                    },
    //!                ),
    //!                H3(
    //!                    Link {
    //!                        pritty: "As an expression",
    //!                        link: "as-as-expression",
    //!                    },
    //!                ),
    //!            ],
    //!        },
    //!        H2 {
    //!            this: Link {
    //!                pritty: "Match",
    //!                link: "match",
    //!            },
    //!            children: [],
    //!        },
    //!        H2 {
    //!            this: Link {
    //!                pritty: "Loops",
    //!                link: "loops",
    //!            },
    //!            children: [
    //!                H3(
    //!                    Link {
    //!                        pritty: "for .. in ..",
    //!                        link: "for-in",
    //!                    },
    //!                ),
    //!                H3(
    //!                    Link {
    //!                        pritty: "while ...",
    //!                        link: "while",
    //!                    },
    //!                ),
    //!                H3(
    //!                    Link {
    //!                        pritty: "loop",
    //!                        link: "loop",
    //!                    },
    //!                ),
    //!            ],
    //!        },
    //!    ],
    //!)
    //!```

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    pub struct PageToc(pub Vec<H2>);

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    pub struct Link {
        // The "nice" name, eg "Creating a book"
        pub pritty: String,
        // The name of the link, eg "creating-a-book"
        pub link: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    pub struct H3(pub Link);

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    pub struct H2 {
        pub this: Link,
        pub children: Vec<H3>,
    }
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
        assert_eq!(content, Content(
            vec![Default::default()]
        ));
    }
}
