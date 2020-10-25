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
    let _conf = config::load(args).context("Failed to load config")?;

    Ok(())
}
