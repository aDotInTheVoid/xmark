use std::path::PathBuf;

use clap::Clap;
use eyre::Result;
use tracing::instrument;

use crate::content::Content;

#[derive(Debug, Clap)]
/// Build a book
pub struct Args {
    #[clap(short, long, default_value = ".")]
    pub(crate) dir: PathBuf,
    #[clap(long)]
    pub(crate) create: bool,
}

impl Args {
    #[instrument]
    pub fn run(self) -> Result<()> {
        let content = Content::load(&self.dir)?;

        dbg!(content);

        Ok(())
    }
}
