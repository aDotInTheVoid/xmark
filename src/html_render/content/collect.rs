
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use eyre::Result;



use super::{Book, Content, Link, Page};

#[derive(Debug, Clone, Default)]
pub struct Dirs {
    pub out_dir: PathBuf,
    pub base_dir: PathBuf,
    pub base_url: String,
}

