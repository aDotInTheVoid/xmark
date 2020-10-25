use std::fs;
use std::path::PathBuf;

use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cli;
use crate::summary::{parse_summary, Summary};

/// The Config as represented in the global xmark.toml
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct GlobalConfigRepr {
    pub books: Vec<String>,
}

/// The config as usable for the programm
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct GlobalConf {
    pub books: Vec<BookConf>,
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BookConf {
    pub location: PathBuf,
    pub summary: Summary,
}

// Convert the disk format to a usable form
pub fn hydrate(gcr: GlobalConfigRepr, args: &cli::Args) -> Result<GlobalConf> {
    Ok(GlobalConf {
        books: gcr
            .books
            .iter()
            .map(|name| {
                let location = args.dir.join(name);
                let summary = parse_summary(&fs::read_to_string(location.join("SUMMARY.md"))?)?;

                Ok(BookConf { location, summary })
            })
            .collect::<Result<_>>()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_gloal_config() {
        let inp = "books = [
            'book-1',
            'book-2',
            'book-3',
            ]
";
        let conf: GlobalConfigRepr = toml::from_str(inp).unwrap();
        assert_eq!(conf.books, vec!["book-1", "book-2", "book-3"]);
    }

    #[test]
    fn hydrate_basic() {
        let args = cli::Args {
            dir: "/home/etc/bax".into(),
            create: false,
        };
        let gcr = GlobalConfigRepr { books: vec![] };
        let gc = GlobalConf { books: vec![] };
        assert_eq!(hydrate(gcr, &args), gc);
    }
}
