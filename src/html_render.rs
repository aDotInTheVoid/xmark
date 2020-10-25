use std::fs;
use std::io::Write;
use std::path::Path;

use eyre::{Context, Result};
use pulldown_cmark::{html, Options, Parser};

use crate::cli;
use crate::config::BookConf;
use crate::summary::Chapter;

pub fn render(book: BookConf, args: &cli::Args) -> Result<()> {
    let out_dir = args.dir.clone().join("_out").join("html");
    fs::create_dir_all(&out_dir)?;

    for i in book.summary.prefix_chapters {
        render_chap_io(&i, &out_dir, &args.dir)?;
    }
    for i in book.summary.suffix_chapters {
        render_chap_io(&i, &out_dir, &args.dir)?;
    }
    for i in book.summary.numbered_chapters {
        i.try_map(|chap| render_chap_io(chap, &out_dir, &args.dir))?;
    }

    Ok(())
}

fn render_chap_io(chapter: &Chapter, build_dir: &Path, base_dir: &Path) -> Result<()> {
    // Not a draft
    if let Some(ref loc) = chapter.location {
        let content = fs::read_to_string(loc)?;
        let html = render_chap(&content);

        let mut path = build_dir.join(loc.strip_prefix(base_dir).expect("Unreachble"));

        path.set_extension("html");

        fs::create_dir_all(path.parent().unwrap())?;

        let mut file =
            fs::File::create(&path).wrap_err_with(|| format!("Failed to create {:?}", &path))?;
        file.write_all(html.as_bytes())?;
    }

    Ok(())
}

// Pure inner for testing
fn render_chap(content: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(content, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet};
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
            create: false,
            dir: temp.path().to_owned(),
        };
        let conf = config::load(&args).unwrap();
        for book in conf.books {
            html_render::render(book, &args).unwrap()
        }


        // BTree so it's in order.
        let paths: BTreeSet<_> = ignore::Walk::new(temp.path())
            .filter_map(|x| x.ok().map(|y| y.into_path()))
            .map(|x| x.into_os_string().into_string().unwrap())
            .collect();

        assert_yaml_snapshot!(paths, {"[]" => dynamic_redaction(move |val, _| {
            val.as_str().unwrap().replace(temp.path().as_os_str().to_str().unwrap(), "")
        })});
    }
}
