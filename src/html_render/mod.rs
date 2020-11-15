use std::fs;
use std::io::Write;

use eyre::{Context, Result};
use pulldown_cmark::{html, Options, Parser};
use tera::Tera;

use crate::cli;
use crate::config::GlobalConf;

use self::content::Content;

pub mod content;

/// Singleton
#[derive(Clone, Debug)]
pub struct HTMLRender<'a> {
    content: Content,
    args: &'a cli::Args,
    inner: HTMLRenderInner,
}

impl<'a> HTMLRender<'a> {
    pub fn new(conf: &GlobalConf, args: &'a cli::Args) -> Result<Self> {
        let dirs = content::Dirs::new(conf, args);
        let content = content::Content::new(conf, &dirs)?;

        let inner = HTMLRenderInner::new().unwrap();

        Ok(Self {
            content,
            args,
            inner,
        })
    }

    pub fn render(&self) -> Result<()> {
        //TODO: Rayon
        for book in &self.content.0 {
            for page in &book.pages {
                let content = fs::read_to_string(&page.input)?;
                let html = self.inner.render_chap(&content, &page.name)?;
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
#[derive(Debug, Clone)]
struct HTMLRenderInner {
    templates: Tera,
}

impl HTMLRenderInner {
    // pub fn from_templates(mut templates: tera::Tera) -> Self {
    //     // Is this right, I dont know.
    //     // Also this is duped here and in Self::new
    //     // TODO
    //     // If the mdconverting is in the template, is can use `safe`
    //     templates.autoescape_on(vec![]);
    //     Self { templates }
    // }

    // When iterating on templates, comment this out so you don't need to rebuild bins
    // TODO: Use rust-embed to solve this
    pub fn new() -> Result<Self> {
        let mut templates: Tera = Default::default();
        templates.add_raw_templates(vec![
            ("head.html", include_str!("../../www/head.html")),
            ("base.html", include_str!("../../www/base.html")),
            ("chapter.html", include_str!("../../www/chapter.html")),
        ])?;
        templates.autoescape_on(vec![]);

        Ok(Self { templates })
    }

    pub fn render_chap(&self, content: &str, name: &str) -> Result<String> {
        let mut context = tera::Context::new();
        let html = render_markdown(content);
        context.insert("mdcontent", &html);
        context.insert("title", name);
        // Needed for Error conversion
        let res = self.templates.render("chapter.html", &context)?;
        Ok(res)
    }
}

// TODO: A million customizations
pub(crate) fn render_markdown(content: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(content, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
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
