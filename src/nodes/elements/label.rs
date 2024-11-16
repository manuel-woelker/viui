use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::nodes::types::NodeProps;
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::result::ViuiResult;
use bevy_reflect::Reflect;

pub struct LabelElement {}

impl Element for LabelElement {
    const NAME: &'static str = "label";
    type State = ();
    type Props = LabelElementProps;
    type Events = NoEvents;
    fn render_element(
        render_context: &mut RenderContext,
        _state: &Self::State,
        props: &Self::Props,
    ) {
        render_context.add_command(RenderCommand::Translate { x: 10.0, y: 25.0 });
        render_context.add_command(RenderCommand::DrawText(props.label.clone()));
    }

    fn layout_element(
        _layout_context: &mut LayoutContext,
        _state: &mut Self::State,
        _props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::FixedLayout {
            width: 200.0,
            height: 40.0,
        })
    }
}

#[derive(Default, Reflect, Debug)]
pub struct LabelElementProps {
    pub label: String,
}

impl NodeProps for LabelElementProps {}
