use std::fs;
use std::path::PathBuf;

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::cli;
use crate::create_missing::create_missing;
use crate::summary::{parse_summary, Chapter, Summary};

/// The Config as represented in the global xmark.toml
#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Default)]
pub struct GlobalConfigRepr {
    pub books: Vec<String>,
    #[serde(default)]
    pub html: HtmlConf,
}

/// The config as usable for the programm
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize, Default)]
pub struct GlobalConf {
    pub books: Vec<Book>,
    pub html: HtmlConf,
}

// An book.
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub location: PathBuf,
    pub summary: Summary,
}

#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize, Default, Eq)]
// https://doc.rust-lang.org/1.47.0/cargo/reference/specifying-dependencies.html#development-dependencies
// Cargo uses kebab, and so shall we
#[serde(default, rename_all = "kebab-case")]
pub struct HtmlConf {
    site_url: Option<String>,
}

pub fn load(args: &cli::Args) -> Result<GlobalConf> {
    let conf = fs::read_to_string(args.dir.clone().join("xmark.toml"))
        .with_context(|| "Couldn't find xmark.toml")?;
    let conf: GlobalConfigRepr = toml::from_str(&conf)?;

    hydrate(conf, &args)
}

// Convert the disk format to a usable form
fn hydrate(gcr: GlobalConfigRepr, args: &cli::Args) -> Result<GlobalConf> {
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

                Ok(Book { location, summary })
            })
            .collect::<Result<_>>()?,
        html: gcr.html,
    })
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;

    use crate::test_utils::manifest_dir_redacter;

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
        assert_eq!(
            conf,
            GlobalConfigRepr {
                books: (&["book-1", "book-2", "book-3"])
                    .iter()
                    .copied()
                    .map(String::from)
                    .collect(),
                html: HtmlConf { site_url: None }
            }
        );

        let inp = "books = []\n[html]";
        let conf: GlobalConfigRepr = toml::from_str(inp).unwrap();
        assert_eq!(conf, Default::default());

        let inp = "books = []\n[html]\nsite-url=\"book\"";
        let conf: GlobalConfigRepr = toml::from_str(inp).unwrap();
        assert_eq!(
            conf,
            GlobalConfigRepr {
                html: HtmlConf {
                    site_url: Some("book".into())
                },
                ..Default::default()
            }
        )
    }

    #[test]
    fn hydrate_basic() {
        let args = cli::Args {
            dir: "/home/etc/bax".into(),
            ..Default::default()
        };
        let gcr = GlobalConfigRepr {
            ..Default::default()
        };
        let gc = GlobalConf {
            ..Default::default()
        };
        assert_eq!(hydrate(gcr, &args).unwrap(), gc);
    }

    #[test]
    fn hydrate_dummy() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dummy-book");
        let args = cli::Args {
            dir,
            ..Default::default()
        };
        let conf = load(&args).unwrap();
        assert_yaml_snapshot!(conf, {
            ".location" => insta::dynamic_redaction(manifest_dir_redacter),
            ".**.location" => insta::dynamic_redaction(manifest_dir_redacter),
        });
    }
}
