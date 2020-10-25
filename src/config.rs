use std::fs;
use std::path::PathBuf;

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::cli;
use crate::create_missing::create_missing;
use crate::summary::{parse_summary, Chapter, Summary};

/// The Config as represented in the global xmark.toml
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct GlobalConfigRepr {
    pub books: Vec<String>,
}

/// The config as usable for the programm
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub struct GlobalConf {
    pub books: Vec<BookConf>,
}

#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub struct BookConf {
    pub location: PathBuf,
    pub slug: String,
    pub summary: Summary,
}

pub fn load(args: &cli::Args) -> Result<GlobalConf> {
    let conf = fs::read_to_string(args.dir.clone().join("xmark.toml"))
        .with_context(|| "Couldn't find xmark.toml")?;
    let conf: GlobalConfigRepr = toml::from_str(&conf)?;

    hydrate(conf, &args)
}

// Convert the disk format to a usable form
pub fn hydrate(gcr: GlobalConfigRepr, args: &cli::Args) -> Result<GlobalConf> {
    Ok(GlobalConf {
        books: gcr
            .books
            .iter()
            .map(|name| {
                let location = args.dir.join(name);
                let summary_location = location.join("SUMMARY.md");
                let mut summary = parse_summary(
                    &fs::read_to_string(&summary_location)
                        .wrap_err_with(|| format!("Couldn't open {:?}", summary_location))?,
                )?;

                if args.create {
                    create_missing(&location, &summary)?;
                }

                let fix_chap_loc = |chap: &mut Chapter| {
                    if let Some(loc) = chap.location.as_deref() {
                        chap.location = Some(location.join(loc));
                    }
                };

                summary.prefix_chapters.iter_mut().for_each(fix_chap_loc);

                summary.suffix_chapters.iter_mut().for_each(fix_chap_loc);

                summary.numbered_chapters.iter_mut().for_each(|chap| {
                    chap.map_mut(fix_chap_loc);
                });

                Ok(BookConf {
                    location,
                    summary,
                    slug: name.to_owned(),
                })
            })
            .collect::<Result<_>>()?,
    })
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;

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
        assert_eq!(hydrate(gcr, &args).unwrap(), gc);
    }

    #[test]
    fn hydrate_dummy() {
        // I don't know why I need the anotation
        let redacter = |mut val, _: insta::internals::ContentPath| {
            // TODO: Dont do insta crimes.
            // This is in #[doc(Hidden)] internals.
            while let insta::internals::Content::Some(some) = val {
                val = *some;
            }

            if let insta::internals::Content::String(s) = val {
                s.replace(dbg!(env!("CARGO_MANIFEST_DIR")), "BASEDIR")
            } else {
                "".into()
            }
        };

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dummy-book");
        let args = cli::Args { dir, create: false };
        let conf = load(&args).unwrap();
        assert_yaml_snapshot!(conf, {
            ".location" => insta::dynamic_redaction(redacter),
            ".**.location" => insta::dynamic_redaction(redacter),
        });
    }
}
