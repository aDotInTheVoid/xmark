use clap::Clap;
use std::path;

#[derive(Clap, Debug)]
pub struct Args {
    #[clap(short, long, default_value=".")]
    pub dir: path::PathBuf
}