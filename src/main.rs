use std::fs;

pub mod config;
pub mod cli;

use eyre::Result;
use clap::Clap;
use std::path;


fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse();
    let conf = fs::read_to_string(args.dir.join("xmark.toml"))?;
    let conf: config::GlobalConfig = toml::from_str(&conf)?;

    
    Ok(())
}
