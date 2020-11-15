use crate::config::{self, Book as CBook};
use crate::summary;
use eyre::Result;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The content in a suitable form.

#[derive(Clone, Debug)]
pub struct ContentOld<'a> {
    pub books: &'a [CBook],
}

impl<'a> ContentOld<'a> {
    pub fn new(books: &'a [CBook]) -> Self {
        Self { books }
    }
}

#[derive(Debug, Clone)]
pub struct Content(Vec<Book>);

#[derive(Clone, Debug)]
struct Book {
    title: String,
    pages: Vec<Page>,
}

#[derive(Debug, Clone)]
pub struct Page {
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

pub struct Dirs<'a> {
    out_dir: &'a Path,
    base_dir: &'a Path,
}

impl Content {
    pub fn new(config: &config::GlobalConf, dirs: Dirs) -> Result<Self> {
        Ok(Self(
            config
                .books
                .iter()
                .map(|x| Book::new(x, &dirs))
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
        for i in &book.summary.prefix_chapters {
            pages_parts.push(Chapter(i));
        }

        let mut out = Vec::<Page>::new();

        let mut heirachy = Vec::new();
        for i in pages_parts {
            match i {
                PageListParts::Chapter(chap) => {
                    let input = match &chap.location {
                        // Skip over the drafts, as they only show up in big toc.
                        None => continue,
                        Some(l) => l,
                    }
                    .clone();

                    let heirachy = heirachy.clone();

                    let output = output_loc(&input, dirs.out_dir, dirs.base_dir)?;
                    let mut page = Page {
                        input,
                        output,
                        // TODO: I think this is the wrong design, as toc can't
                        // be determined utill we read the file, which we arn't
                        // doing here.
                        toc: Default::default(),
                        prev: None,
                        next: None,
                        heirachy,
                    };
                    page.heirachy.push(page.heirachy_element());
                    out.push(page)
                }
                PageListParts::StartSection => {
                    heirachy.push(out.last().unwrap().heirachy_element());
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
        for i in 0..out.len() - 1 {
            let before_url = out[i].url();
            let after_url = out[i + 1].url();
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
            Self::capture_raw_parts(link, out);
            out.push(EndSection)
        }
    }
}

impl Page {
    pub fn heirachy_element(&self) -> Link {
        todo!()
    }

    pub fn url(&self) -> String {
        todo!()
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
#[derive(Debug, Clone)]
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
    #[derive(Debug, Clone, Default)]
    pub struct PageToc(pub Vec<H2>);

    #[derive(Debug, Clone)]
    pub struct Link {
        // The "nice" name, eg "Creating a book"
        pub pritty: String,
        // The name of the link, eg "creating-a-book"
        pub link: String,
    }

    #[derive(Debug, Clone)]
    pub struct H3(pub Link);

    #[derive(Debug, Clone)]
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
