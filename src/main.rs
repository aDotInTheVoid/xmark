pub mod cli;
pub mod config;
mod create_missing;
pub mod html_render;
pub mod summary;
#[cfg(test)]
mod test_utils;

use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse()?;
    let conf = config::load(&args).context("Failed to load config")?;
    let render = html_render::HTMLRender::new(&conf.books, &args);

    render.render()?;

    Ok(())
}
