use crate::nodes::elements::kind::Element;
use crate::nodes::events::NodeEvent;
use crate::nodes::types::NodeProps;
use crate::render::command::RenderCommand;
use bevy_reflect::Reflect;

pub struct LabelElement {}

impl Element for LabelElement {
    const NAME: &'static str = "label";
    type State = ();
    type Props = LabelElementProps;

    fn handle_event(_event: &NodeEvent, _state: &mut Self::State, _props: &Self::Props) {}

    fn render_element(
        render_queue: &mut Vec<RenderCommand>,
        _state: &Self::State,
        props: &Self::Props,
    ) {
        render_queue.push(RenderCommand::Translate { x: 10.0, y: 20.0 });
        render_queue.push(RenderCommand::DrawText(props.label.clone()));
    }
}

#[derive(Default, Reflect, Debug)]
pub struct LabelElementProps {
    pub label: String,
}

impl NodeProps for LabelElementProps {}
