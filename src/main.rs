// SPDX-License-Identifier: GPL-3.0-only
pub(crate) mod cli;
pub(crate) mod content;
pub(crate) mod html_render;
pub(crate) mod render;

#[cfg(test)]
mod test_utils;

use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    cli::init()?;

    let args = cli::Args::parse()?;
    let conf = cli::config::load(&args).context("Failed to load config")?;
    let render = html_render::HTMLRender::new(&conf, &args)?;

    render.render()?;

    Ok(())
}
