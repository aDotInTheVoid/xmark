use clap::Clap;
use eyre::Result;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct Args {
    pub dir: PathBuf,
    pub create: bool,
    pub templates: Option<PathBuf>,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let ArgsInner {
            mut dir,
            create,
            mut templates,
        } = ArgsInner::parse();
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
