use clap::Clap;
use eyre::Result;

use std::path;

#[derive(Debug)]
pub struct Args {
    pub dir: path::PathBuf,
    pub create: bool,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let ArgsInner { mut dir, create } = ArgsInner::parse();
        dir = dir.canonicalize()?;
        Ok(Args { dir, create })
    }
}

#[derive(Clap, Debug)]
struct ArgsInner {
    #[clap(short, long, default_value = ".")]
    pub dir: path::PathBuf,
    #[clap(long)]
    pub create: bool,
}
