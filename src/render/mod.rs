use std::borrow::Cow;
use std::path::PathBuf;

use eyre::Result;

pub trait Render {
    fn name() -> Cow<'static, str>;

    fn create(conf: &Conf) -> Self;

    fn run(self) -> Result<()>;
}

pub struct Conf {
    pub out_dir: PathBuf,
    pub aux_dir: PathBuf,
}
