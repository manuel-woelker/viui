use crate::nodes::events::InputEvent;
use crate::nodes::types::{NodeEvents, NodeProps, NodeState};
use crate::render::command::RenderCommand;

pub type EventTrigger<'a> = dyn FnMut(&str) + 'a;

pub trait Element {
    const NAME: &'static str;
    type State: NodeState + Default;
    type Props: NodeProps + Default;
    type Events: NodeEvents;

    fn handle_event(
        _event: &InputEvent,
        _state: &mut Self::State,
        _props: &Self::Props,
        _event_trigger: &mut EventTrigger<'_>,
    ) {
    }
    fn render_element(
        render_queue: &mut Vec<RenderCommand>,
        state: &Self::State,
        props: &Self::Props,
    );
}

impl NodeEvents for () {}
