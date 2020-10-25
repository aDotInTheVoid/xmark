use crate::summary::{Link, Summary};
use eyre::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use log::debug;

pub fn create_missing(src_dir: &Path, summary: &Summary) -> Result<()> {
    let mut items: Vec<_> = summary
        .prefix_chapters
        .iter()
        .chain(summary.suffix_chapters.iter())
        .map(|chapter| Link {
            // This is like 3 strings
            chapter: chapter.clone(),
            ..Default::default()
        })
        //TODO: Don't clone, it can be quite nested
        .chain(summary.numbered_chapters.iter().cloned())
        .collect();

    while !items.is_empty() {
        let link = items.pop().expect("already checked");

        if let Some(ref location) = link.chapter.location {
            let filename = src_dir.join(location);
            if !filename.exists() {
                if let Some(parent) = filename.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                debug!("Creating missing file {}", filename.display());

                let mut f = File::create(&filename)?;
                writeln!(f, "# {}", link.chapter.name)?;
            }
        }

        items.extend(link.nested_items);
    }

    Ok(())
}
