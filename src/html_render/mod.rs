// SPDX-License-Identifier: GPL-3.0-only
use std::fs;
use std::io::Write;

use eyre::{Context, Result};

use crate::cli;
use crate::config::GlobalConf;

use self::content::{Content, Page};

pub mod content;

/// Singleton
pub struct HTMLRender<'a> {
    content: Content,
    // I'll need em later, when this gets fancy
    _args: &'a cli::Args,
    inner: HTMLRenderInner,
}

impl<'a> HTMLRender<'a> {
    pub fn new(conf: &GlobalConf, args: &'a cli::Args) -> Result<Self> {
        let dirs = content::Dirs::new(conf, args);
        let content = content::Content::new(conf, &dirs)?;

        let inner = HTMLRenderInner::new().unwrap();

        Ok(Self {
            content,
            _args: args,
            inner,
        })
    }

    pub fn render(&self) -> Result<()> {
        //TODO: Rayon
        for book in &self.content.0 {
            for page in &book.pages {
                let html = self.inner.render_page(page)?;
                fs::create_dir_all(page.output.parent().unwrap())?;
                let mut file = fs::File::create(&page.output)
                    .wrap_err_with(|| format!("Failed to create {:?}", &page.output))?;
                file.write_all(html.as_bytes())?;
            }
        }
        Ok(())
    }
}

// An abstraction of HTMLRender that does no io.
// TODO: we do IO here now, so why does this exist
struct HTMLRenderInner {
    templates: ramhorns::Ramhorns,
}

impl HTMLRenderInner {
    // TODO: Use rust-embed.
    pub fn new() -> Result<Self> {
        let templates = ramhorns::Ramhorns::from_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/www/")).unwrap();

        Ok(Self { templates })
    }

    pub fn render_page(&self, page: &Page) -> Result<String> {
        let rp = self::content::render::Page::new(page)?;
        let tpl = self.templates.get("page.html").unwrap();
        // TODO: Use render_to_file or something
        Ok(tpl.render(&rp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use assert_fs::prelude::*;
    use insta::{assert_yaml_snapshot, dynamic_redaction};

    use crate::{cli, config, html_render};

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

    #[test]
    fn html_inner_includes() {
        let x = HTMLRenderInner::new();
        assert!(x.is_ok());
    }
}
