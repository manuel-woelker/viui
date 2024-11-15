use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::render::command::RenderCommand;
use crate::result::ViuiResult;

pub struct HStackElement {}

impl Element for HStackElement {
    const NAME: &'static str = "hstack";
    type State = ();
    type Props = ();
    type Events = NoEvents;
    fn render_element(
        _render_queue: &mut Vec<RenderCommand>,
        _state: &Self::State,
        _props: &Self::Props,
    ) {
    }

    fn layout_element(_state: &Self::State, _props: &Self::Props) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::HorizontalLayout {})
    }
}
