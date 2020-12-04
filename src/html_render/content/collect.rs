
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use eyre::Result;

use crate::config::GlobalConf;
use crate::{cli, config, summary};

use super::{Book, Content, Link, Page};