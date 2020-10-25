use std::fs;
use std::path::PathBuf;

pub mod book;
pub mod cli;
pub mod config;
mod create_missing;
pub mod summary;

use clap::Clap;
use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();
    dbg!(&args);
    let conf = fs::read_to_string(args.dir.join("xmark.toml"))
        .with_context(|| "Couldn't find xmark.toml")?;
    let conf: config::GlobalConfigRepr = toml::from_str(&conf)?;

    let conf = config::hydrate(conf, &args)?;

    dbg!(conf);

    Ok(())
}
