use std::path::PathBuf;



#[derive(Debug, Clone, Default)]
pub struct Dirs {
    pub out_dir: PathBuf,
    pub base_dir: PathBuf,
    pub base_url: String,
}

