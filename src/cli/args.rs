use clap::Clap;
use eyre::Result;
use tracing::instrument;

#[derive(Clap, Debug)]
pub(super) struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub(super) enum SubCommand {
    Build(super::cmd::build::Args), // TODO: init, clean, watch, serve
}

impl Args {
    #[instrument]
    pub fn run(self) -> Result<()> {
        match self.subcmd {
            SubCommand::Build(c) => c.run(),
        }
    }
}
