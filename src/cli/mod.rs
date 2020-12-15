// SPDX-License-Identifier: GPL-3.0-only
use clap::Clap;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::instrument;

pub mod config;
pub mod summary;

mod create_missing;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Args {
    pub dir: PathBuf,
    pub create: bool,
    pub templates: Option<PathBuf>,
}

impl Args {
    #[instrument]
    pub fn parse() -> Result<Self> {
        Self::parse_from(ArgsInner::parse())
    }

    #[instrument]
    fn parse_from(inner: ArgsInner) -> Result<Self> {
        let ArgsInner {
            mut dir,
            create,
            mut templates,
        } = inner;
        dir = dir.canonicalize()?;
        templates = match templates {
            Some(p) => Some(p.canonicalize()?),
            None => None,
        };
        Ok(Args {
            dir,
            create,
            templates,
        })
    }
}

#[instrument]
pub fn init() -> Result<()> {
    color_eyre::install()?;

    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

#[derive(Clap, Debug)]
struct ArgsInner {
    #[clap(short, long, default_value = ".")]
    pub dir: PathBuf,
    #[clap(long)]
    pub create: bool,
    #[clap(short, long)]
    pub templates: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use insta::{assert_yaml_snapshot, dynamic_redaction};

    use crate::test_utils::manifest_dir_redacter;

    use super::*;
    #[test]
    fn canonicalize_cwd_dir() {
        let args = ArgsInner {
            dir: ".".into(),
            create: true,
            templates: None,
        };
        let args = Args::parse_from(args).unwrap();
        // Yaml uses ~ for null
        assert_yaml_snapshot!(args, {
            ".*" => dynamic_redaction(manifest_dir_redacter)
        });
    }

    #[test]
    fn canonicalize_rel_templ() {
        let args = ArgsInner {
            dir: ".".into(),
            create: true,
            // This needs to exist, as `.canonicalize` will read, in case it's a symblink
            templates: Some("./www".into()),
        };
        let args = Args::parse_from(args).unwrap();
        assert_yaml_snapshot!(args, {
            ".*" => dynamic_redaction(manifest_dir_redacter)
        });
    }

    #[test]
    fn absolute() {
        let args = ArgsInner {
            // This should exist on most non windows systems
            // TODO: win
            dir: "/usr".into(),
            create: false,
            templates: None,
        };
        let args = Args::parse_from(args).unwrap();
        assert_yaml_snapshot!(args, {
            ".*" => dynamic_redaction(manifest_dir_redacter)
        });
    }
}
