use crate::nodes::events::NodeEvent;
use crate::nodes::types::{NodeProps, NodeState};
use crate::render::command::RenderCommand;

pub trait Element {
    const NAME: &'static str;
    type State: NodeState + Default;
    type Props: NodeProps + Default;

    fn handle_event(event: &NodeEvent, state: &mut Self::State, props: &Self::Props);
    fn render_element(
        render_queue: &mut Vec<RenderCommand>,
        state: &Self::State,
        props: &Self::Props,
    );
}
