
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use eyre::Result;

use crate::config::GlobalConf;
use crate::{cli, config, summary};

use super::{Book, Content, Link, Page};

#[derive(Debug, Clone, Default)]
pub struct Dirs {
    pub out_dir: PathBuf,
    pub base_dir: PathBuf,
    pub base_url: String,
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
