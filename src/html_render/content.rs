use crate::config::{self, Book as CBook};
use crate::summary;
use eyre::Result;
use std::path::PathBuf;

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
    /// The md input file. None => draft
    pub input: Option<PathBuf>,
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

impl Content {
    pub fn new(config: &config::GlobalConf) -> Result<Self> {
        Ok(Self(
            config.books.iter().map(Book::new).collect::<Result<_>>()?,
        ))
    }
}

impl Book {
    pub fn new(book: &config::Book) -> Result<Self> {
        let title = book.summary.title.clone();
        let pages = Self::capture_pages(book)?;

        Ok(Self { title, pages })
    }

    fn capture_pages(book: &config::Book) -> Result<Vec<Page>> {
        use PageListParts::*;

        // We need to hold onto a bungh of stuff as we walk the tree, ands its
        // nicer if thats a list, and we just preserve the tree structure by
        // saying when we go up and down.

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

        todo!()
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
    #[derive(Debug, Clone)]
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
