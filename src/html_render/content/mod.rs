
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
    pub toc: (),
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
