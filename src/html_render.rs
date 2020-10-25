use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use eyre::{Context, Result};
use pulldown_cmark::{Parser, Options, html};

use crate::config::BookConf;
use crate::cli;
use crate::summary::Chapter;

pub fn render(book: BookConf, args: &cli::Args) -> Result<()> {
    let out_dir = args.dir.clone().join("_out").join("html").join(book.slug);
    fs::create_dir_all(&out_dir)?;

    for i in book.summary.prefix_chapters {
        render_chap_io(&i, &out_dir)?;
    }
    for i in book.summary.suffix_chapters {
        render_chap_io(&i, &out_dir)?;
    }
    for i in book.summary.numbered_chapters {
        i.try_map(|chap| render_chap_io(chap, &out_dir))?;
    }

    Ok(())
}

fn render_chap_io(chapter: &Chapter, base_dir: &Path) -> Result<()> {
    // Not a draft
    if let Some(ref loc) = chapter.location {
        let dup: PathBuf = loc.components().take(2).collect();
        let content = fs::read_to_string(loc)?;
        let html = render_chap(&content);

        let mut path = base_dir.to_owned().join(loc.strip_prefix(dup).expect("Unreachble"));
        path.set_extension("html");

        fs::create_dir_all(path.parent().unwrap())?;

        let mut file = fs::File::create(&path).wrap_err_with(||format!("Failed to create {:?}", &path))?;
        file.write_all(html.as_bytes())?;
    }

    Ok(())
}

// Pure inner for testing
fn render_chap(content: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(content   , opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}