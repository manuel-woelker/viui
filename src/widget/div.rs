use crate::eval::tree::Properties;
use crate::render::context::RenderContext;
use crate::result::ViuiResult;
use crate::widget::Widget;

#[derive(Default)]
pub struct DivWidget {}

impl Widget for DivWidget {
    fn render(&self, _render_context: &mut RenderContext, _props: &Properties) -> ViuiResult<()> {
        Ok(())
    }
}
