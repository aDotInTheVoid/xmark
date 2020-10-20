use std::fs;

pub mod book;
pub mod cli;
pub mod config;
pub mod summary;

use clap::Clap;
use eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();
    let conf = fs::read_to_string(args.dir.join("xmark.toml"))?;
    let conf: config::GlobalConfig = toml::from_str(&conf)?;

    Ok(())
}
