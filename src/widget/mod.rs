use crate::eval::tree::Properties;
use crate::ir::node::WidgetFactory;
use crate::render::context::RenderContext;
use crate::result::ViuiResult;

pub mod div;
pub mod label;

pub trait Widget: Send {
    fn render(&self, render_context: &mut RenderContext, props: &Properties) -> ViuiResult<()>;
}

impl<T: Widget + Default + 'static> WidgetFactory for T {
    fn create_widget(&self) -> ViuiResult<Box<dyn Widget>> {
        Ok(Box::new(Self::default()))
    }
}
