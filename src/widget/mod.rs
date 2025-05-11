use crate::render::context::RenderContext;
use crate::result::ViuiResult;

pub mod label;

pub trait Widget {
    fn render(&self, render_context: &mut RenderContext) -> ViuiResult<()>;
}
