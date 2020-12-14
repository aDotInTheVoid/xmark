// SPDX-License-Identifier: GPL-3.0-only
use super::summary::{Link, Summary};
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use assert_fs::prelude::*;
    use insta::{assert_yaml_snapshot, dynamic_redaction};

    use crate::cli::{self, config};
    use crate::html_render;

    #[test]
    fn dummy_e2e() {
        let temp = assert_fs::TempDir::new().unwrap();

        temp.copy_from(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dummy-book"),
            &["xmark.toml", "*/SUMMARY.md"],
        )
        .unwrap();

        let args = cli::Args {
            dir: temp.path().to_owned(),
            create: true,
            ..Default::default()
        };

        let conf = config::load(&args).unwrap();
        let render = html_render::HTMLRender::new(&conf, &args).unwrap();
        render.render().unwrap();

        // BTree so it's in order.
        let paths: BTreeSet<_> = ignore::Walk::new(temp.path())
            .filter_map(|x| x.ok().map(|y| y.into_path()))
            .map(|x| x.into_os_string().into_string().unwrap())
            .filter(|x| !x.contains("_out"))
            .collect();

        assert_yaml_snapshot!(paths, {"[]" => dynamic_redaction(move |val, _| {
            val.as_str().unwrap().replace(temp.path().as_os_str().to_str().unwrap(), "")
        })});
    }
}
