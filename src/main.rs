pub mod book;
pub mod cli;
pub mod config;
mod create_missing;
pub mod html_render;
pub mod summary;

use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse()?;
    let conf = config::load(&args).context("Failed to load config")?;
    for book in conf.books {
        html_render::render(book, &args)?;
    }

    Ok(())
}
