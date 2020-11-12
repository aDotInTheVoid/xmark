use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use eyre::{Context, Result};
use pulldown_cmark::{html, Options, Parser};
use tera::Tera;

use crate::cli;
use crate::config::Book;
use crate::summary::Chapter;

mod content;

use content::Content;

/// Singleton
#[derive(Clone, Debug)]
pub struct HTMLRender<'a, 'b> {
    content: Content<'b>,
    args: &'a cli::Args,
    out_dir: PathBuf,
    inner: HTMLRenderInner,
}

impl<'a, 'b> HTMLRender<'a, 'b> {
    pub fn new(books: &'b [Book], args: &'a cli::Args) -> Self {
        let out_dir = args.dir.clone().join("_out").join("html");
        let inner = HTMLRenderInner::new().unwrap();
        let content = Content::new(books);

        Self {
            content,
            args,
            out_dir,
            inner,
        }
    }

    pub fn render(&self) -> Result<()> {
        fs::create_dir_all(&self.out_dir)?;

        for book in self.content.books {
            for i in &book.summary.prefix_chapters {
                self.render_chap_io(i)?;
            }
            for i in &book.summary.suffix_chapters {
                self.render_chap_io(i)?;
            }
            for i in &book.summary.numbered_chapters {
                i.try_map(|chap| self.render_chap_io(chap))?;
            }
        }

        Ok(())
    }

    fn render_chap_io(&self, chapter: &Chapter) -> Result<()> {
        // Not a draft
        if let Some(ref loc) = chapter.location {
            let content = fs::read_to_string(loc)?;
            let html = self.inner.render_chap(&content, &chapter.name)?;

            let mut path = self
                .out_dir
                .join(loc.strip_prefix(&self.args.dir).expect("Unreachble"));

            // foo/README.md -> foo/index.html     -> foo/
            // foo/bar.md    -> foo/bar/index.html -> foo/bar
            if path.file_name() == Some(OsStr::new("README.md")) {
                path.set_file_name("index.html")
            } else {
                path.set_extension("");
                path.push("index.html");
            }

            fs::create_dir_all(path.parent().unwrap())?;

            let mut file = fs::File::create(&path)
                .wrap_err_with(|| format!("Failed to create {:?}", &path))?;
            file.write_all(html.as_bytes())?;
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
        let render = html_render::HTMLRender::new(&conf.books, &args);
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
