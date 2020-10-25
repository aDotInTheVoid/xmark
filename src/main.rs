use std::fs;
use std::path::PathBuf;

pub mod book;
pub mod cli;
pub mod config;
mod create_missing;
pub mod summary;
use create_missing::create_missing;

use clap::Clap;
use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();
    dbg!(&args);
    let conf = fs::read_to_string(args.dir.join("xmark.toml"))
        .with_context(|| "Couldn't find xmark.toml")?;
    let conf: config::GlobalConfigRepr = toml::from_str(&conf)?;

    for i in conf.books {
        let summary_path = args.dir.join(i).join("SUMMARY.md");
        let summary = fs::read_to_string(&summary_path)
            .with_context(|| format!("Couldn't open {:?}", &summary_path))?;
        let summary = summary::parse_summary(&summary)?;
        dbg!(summary);
    }

    if args.create {}

    Ok(())
}
