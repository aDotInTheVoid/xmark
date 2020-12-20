use clap::Clap;
use eyre::Result;

use super::args;

pub fn main() -> Result<()> {
    println!("This is XMARK");
    let args = args::Args::parse();
    args.run()?;
    println!("Thats all folks");
    Ok(())
}
