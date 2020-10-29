use clap::Clap;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Args {
    pub dir: PathBuf,
    pub create: bool,
    pub templates: Option<PathBuf>,
}

impl Args {
    pub fn parse() -> Result<Self> {
        Self::parse_from(ArgsInner::parse())
    }

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
    use insta::assert_yaml_snapshot;

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
        // TODO: use the redacter from `hydrate_dummy`
        // This will fail CI
        assert_yaml_snapshot!(args);
    }

    #[test]
    fn canonicalize_rel_templ () {
        let args = ArgsInner {
            dir: ".".into(),
            create: true,
            // This needs to exist, as `.canonicalize` will read, in case it's a symblink
            templates: Some("./www".into()),
        };
        let args = Args::parse_from(args).unwrap();
        assert_yaml_snapshot!(args);
    }

    #[test]
    fn absolute() {
        let args = ArgsInner {
            // This should exist on most non windows systems
            // TODO: win
            dir: "/usr".into(),
            create: false,
            templates: None
        };
        let args = Args::parse_from(args).unwrap();
        assert_yaml_snapshot!(args);  
    }
}
