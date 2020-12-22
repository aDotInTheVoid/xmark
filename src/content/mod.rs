use std::fs;
use std::path::{Path, PathBuf};

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::cli::conf::{self, DiskConf};

pub mod summary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub books: Vec<Book>,
    out_dir: PathBuf,
    aux_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    root_loc: PathBuf,
    name: String,
}

impl Content {
    pub fn load(dir: &Path) -> Result<Self> {
        let dir = dir.canonicalize()?;

        let conf = dir.join("xmark.toml");
        let conf = fs::read_to_string(&conf).with_context(|| format!("Cannot read {:?}", &conf))?;
        let conf: DiskConf = toml::from_str(&conf)?;

        let books = conf
            .books
            .iter()
            .map(|loc| Book::new(loc, &dir))
            .collect::<Result<_, _>>()?;

        let out_dir = dir.join("_out");
        let aux_dir = dir.join("_build");

        Ok(Self {
            books,
            out_dir,
            aux_dir,
        })
    }
}

impl Book {
    fn new(loc: &conf::Location, base_dir: &Path) -> Result<Self> {
        let (name, root_loc) = match loc {
            conf::Location::Bare(name) => (name, name),
            conf::Location::Named { name, root_loc } => (name, root_loc),
        };

        let name = name.to_owned();
        let root_loc = base_dir.join(root_loc);

        Ok(Self { name, root_loc })
    }
}
