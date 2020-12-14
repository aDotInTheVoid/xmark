// SPDX-License-Identifier: GPL-3.0-only
use std::fs;
use std::io::Write;

use eyre::{Context, Result};
use fs_extra::dir as fsx;

use crate::cli;
use crate::cli::config::GlobalConf;

use self::content::{Book, Content, Page};

pub mod content;

/// Singleton
pub struct HTMLRender<'a> {
    content: Content,
    // I'll need em later, when this gets fancy
    _args: &'a cli::Args,
    templates: ramhorns::Ramhorns,
    dirs: content::Dirs,
}

impl<'a> HTMLRender<'a> {
    pub fn new(conf: &GlobalConf, args: &'a cli::Args) -> Result<Self> {
        let dirs = content::Dirs::new(conf, args);

        //TODO: This wount work for incrmental or multi-renderer
        //TODO: Embed static
        if dirs.out_dir.exists() {
            fs::remove_dir_all(&dirs.out_dir)?;
        }

        let parent = dirs.out_dir.parent().unwrap();

        fs::create_dir_all(parent)?;

        fsx::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/www/static/"),
            parent,
            &Default::default(),
        )?;

        fs::rename(parent.join("static"), &dirs.out_dir)?;

        let content = content::Content::new(conf, &dirs)?;

        let templates = ramhorns::Ramhorns::from_folder(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/www/templates/"
        ))?;

        Ok(Self {
            content,
            _args: args,
            templates,
            dirs,
        })
    }

    pub fn render(&self) -> Result<()> {
        //TODO: Rayon
        for book in &self.content.0 {
            for page in &book.pages {
                let html = self.render_page(page, book)?;
                fs::create_dir_all(page.output.parent().unwrap())?;
                let mut file = fs::File::create(&page.output)
                    .wrap_err_with(|| format!("Failed to create {:?}", &page.output))?;
                file.write_all(html.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn render_page(&self, page: &Page, book: &Book) -> Result<String> {
        let rp = self::content::render::Page::new(page, &self, book)?;
        let tpl = self.templates.get("page.html").unwrap();
        // TODO: Use render_to_file or something
        Ok(tpl.render(&rp))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use assert_fs::prelude::*;
    use insta::{assert_yaml_snapshot, dynamic_redaction};

    use crate::{
        cli::{self, config},
        html_render,
    };

    #[test]
    fn dummy_e2e() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dummy-book"),
            &["xmark.toml", "book-*/**"],
        )
        .unwrap();

        let args = cli::Args {
            dir: temp.path().to_owned(),
            ..Default::default()
        };
        let conf = config::load(&args).unwrap();
        let render = html_render::HTMLRender::new(&conf, &args).unwrap();
        render.render().unwrap();

        // BTree so it's in order.
        let paths: BTreeSet<_> = ignore::Walk::new(temp.path())
            .filter_map(|x| x.ok().map(|y| y.into_path()))
            .map(|x| x.into_os_string().into_string().unwrap())
            .collect();

        assert_yaml_snapshot!(paths, {"[]" => dynamic_redaction(move |val, _| {
            val.as_str().unwrap().replace(temp.path().as_os_str().to_str().unwrap(), "")
        })});
    }

    // #[test]
    // fn render_readmes() {
    //     glob!("render_html_tests/*.md", |path| {
    //         let input = fs::read_to_string(path).unwrap();
    //         let out = render_chap(&input);
    //         assert_snapshot!(out);
    //     })
    // }
}
