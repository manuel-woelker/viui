use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::render::context::RenderContext;
use crate::result::ViuiResult;

pub struct HStackElement {}

impl Element for HStackElement {
    const NAME: &'static str = "hstack";
    type State = ();
    type Props = ();
    type Events = NoEvents;
    fn render_element(
        _render_context: &mut RenderContext,
        _state: &Self::State,
        _props: &Self::Props,
    ) {
    }

    fn layout_element(_state: &Self::State, _props: &Self::Props) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::HorizontalLayout {})
    }
}
