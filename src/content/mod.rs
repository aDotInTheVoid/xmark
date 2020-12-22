use std::fs;
use std::path::{Path, PathBuf};

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::cli::conf::{self, DiskConf};

pub mod summary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub books: Vec<Book>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub out_dir: PathBuf,
    pub aux_dir: PathBuf,
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

        Ok(Self { books })
    }
}

impl Book {
    fn new(loc: &conf::Location, base_dir: &Path) -> Result<Self> {
        let out_name = Path::new(match loc {
            conf::Location::Bare(name) => name,
            conf::Location::Named { name, .. } => name,
        });

        let mut out_dir = base_dir.join("_out");
        out_dir.push(out_name);

        let mut aux_dir = base_dir.join("_build");
        aux_dir.push(out_name);

        Ok(Self { out_dir, aux_dir })
    }
}
