use std::fs;
use std::io::Write;
use std::path::PathBuf;

use eyre::{Context, Result};
use pulldown_cmark::{html, Options, Parser};

use crate::cli;
use crate::config::Book;
use crate::summary::Chapter;

/// Singleton
#[derive(Clone, Debug)]
pub struct HTMLRender<'a, 'b> {
    books: &'b [Book],
    args: &'a cli::Args,
    out_dir: PathBuf,
}

impl<'a, 'b> HTMLRender<'a, 'b> {
    pub fn new(books: &'b [Book], args: &'a cli::Args) -> Self {
        let out_dir = args.dir.clone().join("_out").join("html");

        Self {
            books,
            args,
            out_dir,
        }
    }

    pub fn render(&self) -> Result<()> {
        fs::create_dir_all(&self.out_dir)?;

        for book in self.books {
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
            let html = render_chap(&content);

            let mut path = self
                .out_dir
                .join(loc.strip_prefix(&self.args.dir).expect("Unreachble"));

            path.set_extension("html");

            fs::create_dir_all(path.parent().unwrap())?;

            let mut file = fs::File::create(&path)
                .wrap_err_with(|| format!("Failed to create {:?}", &path))?;
            file.write_all(html.as_bytes())?;
        }

        Ok(())
    }
}

// Pure inner for testing
pub(crate) fn render_chap(content: &str) -> String {
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
    use std::fs;
    use std::path::PathBuf;

    use assert_fs::prelude::*;
    use insta::{assert_snapshot, assert_yaml_snapshot, dynamic_redaction, glob};

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
            create: false,
            dir: temp.path().to_owned(),
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

    #[test]
    fn render_readmes() {
        glob!("render_html_tests/*.md", |path| {
            let input = fs::read_to_string(path).unwrap();
            let out = render_chap(&input);
            assert_snapshot!(out);
        })
    }
}
