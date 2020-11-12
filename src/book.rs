// This is getting moved to somewhere, and isnt regesteed asa a module

use std::{
    fmt::{self, Display, Formatter},
    iter::FromIterator,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

/// An In Memory Book
pub struct Book {
    pub home: Page,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// The "display" name. Goes in <h1>
    pub title: String,
    /// The filename
    pub slug: String,
    /// The chapter's contents in raw markdown
    pub content: String,
    /// The chapter's section number, if it has one.
    pub number: Option<SectionNumber>,
    /// Nested items.
    pub sub_items: Vec<Page>,
    /// The chapter's location, relative to the `SUMMARY.md` file.
    pub path: Option<PathBuf>,
    /// An ordered list of the names of each chapter above this one, in the hierarchy.
    pub parent_names: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct SectionNumber(pub Vec<u32>);

impl Display for SectionNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "0")
        } else {
            for item in &self.0 {
                write!(f, "{}.", item)?;
            }
            Ok(())
        }
    }
}

impl Deref for SectionNumber {
    type Target = Vec<u32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SectionNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<u32> for SectionNumber {
    fn from_iter<I: IntoIterator<Item = u32>>(it: I) -> Self {
        SectionNumber(it.into_iter().collect())
    }
}
