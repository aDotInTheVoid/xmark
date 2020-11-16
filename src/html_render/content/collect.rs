use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use eyre::Result;

use crate::config::GlobalConf;
use crate::{cli, config, summary};

use super::{Book, Content, Link, Page};

// Oh dear god the allocations
#[derive(Debug, Clone, Default)]
pub struct Dirs {
    pub out_dir: PathBuf,
    pub base_dir: PathBuf,
    pub base_url: String,
}

impl Dirs {
    pub fn new(conf: &GlobalConf, args: &cli::Args) -> Self {
        Self {
            base_dir: args.dir.clone(),
            out_dir: args.dir.join("_out").join("html"),
            base_url: conf
                .html
                .site_url
                .to_owned()
                .unwrap_or_else(|| "/".to_owned()),
        }
    }
}

impl Content {
    pub fn new(config: &config::GlobalConf, dirs: &Dirs) -> Result<Self> {
        Ok(Self(
            config
                .books
                .iter()
                .map(|x| Book::new(x, dirs))
                .collect::<Result<_>>()?,
        ))
    }
}

impl Book {
    pub fn new(book: &config::Book, dirs: &Dirs) -> Result<Self> {
        let title = book.summary.title.clone();
        let pages = Self::capture_pages(book, dirs)?;

        Ok(Self { title, pages })
    }

    fn capture_pages(book: &config::Book, dirs: &Dirs) -> Result<Vec<Page>> {
        use PageListParts::*;

        // We need to hold onto a bungh of stuff as we walk the tree, ands its
        // nicer if thats a list, and we just preserve the tree structure by
        // saying when we go up and down.

        // Create flat list
        let mut pages_parts = Vec::new();
        for i in &book.summary.prefix_chapters {
            pages_parts.push(Chapter(i));
        }
        for i in &book.summary.numbered_chapters {
            Self::capture_raw_parts(i, &mut pages_parts);
        }
        for i in &book.summary.suffix_chapters {
            pages_parts.push(Chapter(i));
        }

        let mut out = Vec::<Page>::new();
        let mut heirachy = vec![Link {
            prity: book.summary.title.clone(),
            link: Path::new(&dirs.base_url)
                .join(book.location.strip_prefix(&dirs.base_dir)?)
                .into_os_string()
                .into_string()
                .map_err(|x| eyre::eyre!("Invalid string {:?}", x))?,
        }];

        for i in pages_parts {
            match i {
                PageListParts::Chapter(chap) => {
                    let input = match &chap.location {
                        // Skip over the drafts, as they only show up in big toc.
                        None => continue,
                        Some(l) => l,
                    }
                    .clone();

                    let name = chap.name.clone();

                    // This is quite wastefull in terms of allocs, but who cares
                    let heirachy = heirachy.clone();

                    let output = output_loc(&input, &dirs.out_dir, &dirs.base_dir)?;
                    let mut page = Page {
                        input,
                        name,
                        output,
                        heirachy,
                        // TODO: I think this is the wrong design, as toc can't
                        // be determined utill we read the file, which we arn't
                        // doing here.
                        toc: Default::default(),
                        prev: None,
                        next: None,
                    };
                    page.heirachy.push(page.heirachy_element(dirs)?);
                    out.push(page)
                }
                PageListParts::StartSection => {
                    heirachy.push(out.last().unwrap().heirachy_element(dirs)?);
                }
                PageListParts::EndSection => {
                    heirachy.pop();
                }
            }
        }
        // We can't have something nice like
        // for i in out.windows_mut(2) {
        //     let before = &i[0];
        //     let after = &i[1];
        //     before.next = Some(after.url());
        //     after.prev = Some(before.url());
        // }
        // Because GAT's.
        for i in 0..out.len().saturating_sub(1) {
            let before_url = out[i].url(dirs)?;
            let after_url = out[i + 1].url(dirs)?;
            out[i].next = Some(after_url);
            out[i + 1].prev = Some(before_url);
        }

        Ok(out)
    }

    fn capture_raw_parts<'a>(link: &'a summary::Link, out: &mut Vec<PageListParts<'a>>) {
        use PageListParts::*;

        out.push(Chapter(&link.chapter));
        if !link.nested_items.is_empty() {
            out.push(StartSection);
            for i in &link.nested_items {
                Self::capture_raw_parts(i, out);
            }
            out.push(EndSection)
        }
    }
}

impl Page {
    pub fn heirachy_element(&self, dirs: &Dirs) -> Result<Link> {
        Ok(Link {
            prity: self.name.clone(),
            link: self.url(dirs)?,
        })
    }

    pub fn url(&self, dirs: &Dirs) -> Result<String> {
        let relative_pos = self.output.strip_prefix(&dirs.out_dir)?;
        let mut url = Path::new(&dirs.base_url).join(relative_pos);
        url.pop();
        Ok(
            url.into_os_string()
                .into_string()
                .map_err(|x| eyre::eyre!("Invalid string {:?}", x))?
                .replace("/./", "/"), // Hack
        )
    }
}
/// Fun helper type
///
/// 1. Foo
/// 2. Bar
/// 2.1. Baz
/// 2.1.1 Quix
/// 2.2 Spam
///
/// Chapter(Foo)
/// Chapter(Bar)
/// StartSection
/// Chapter(Baz)
/// StartSection
enum PageListParts<'a> {
    //TODO: A better name
    Chapter(&'a summary::Chapter),
    StartSection,
    EndSection,
}

pub fn output_loc(input_loc: &Path, out_dir: &Path, base_dir: &Path) -> Result<PathBuf> {
    let mut path = out_dir.join(input_loc.strip_prefix(base_dir)?);
    if path.file_name() == Some(OsStr::new("README.md")) {
        path.set_file_name("index.html")
    } else {
        path.set_extension("");
        path.push("index.html");
    }
    Ok(path)
}
