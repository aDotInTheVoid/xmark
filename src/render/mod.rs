use eyre::Result;

use crate::cli::config::{GlobalConf, HtmlConf};
use crate::content::Content;
use crate::html_render::HTMLRender;
use crate::cli::{Args};

pub(crate) struct HTML<'a>(HTMLRender<'a>);


pub(crate) struct GlobalRenderContext {
    conf: GlobalConf,
    args: Args,
    content: Content
}

pub(crate) trait Renderer  {
    type LocalContext;

    fn new(gc: &GlobalConf, lc: &Self::LocalContext) -> Self;

    fn render(&self) -> Result<()>;
}

impl Renderer for HTML<'_> {

    type LocalContext = HtmlConf;

    fn new(gc: &GlobalConf, lc: &HtmlConf) -> Self {
        todo!()
    }

    fn render(&self) -> Result<()> {
        todo!()
    }
}
