
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub mod collect;
pub mod pagetoc;
pub use collect::Dirs;



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
    pub heirachy: ()
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
