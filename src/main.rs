// SPDX-License-Identifier: GPL-3.0-only
pub mod cli;
pub mod content;
pub mod html_render;
#[cfg(test)]
mod test_utils;

use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Args::parse()?;
    let conf = cli::config::load(&args).context("Failed to load config")?;
    let render = html_render::HTMLRender::new(&conf, &args)?;

    render.render()?;

    Ok(())
}
