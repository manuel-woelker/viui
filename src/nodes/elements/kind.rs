use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::events::InputEvent;
use crate::nodes::types::{NodeEvents, NodeProps, NodeState};
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use crate::types::Float;
use bevy_reflect::Reflect;

pub type EventTrigger<'a, E> = dyn FnMut(E) + 'a;

pub trait Element {
    const NAME: &'static str;
    type State: NodeState + Default;
    type Props: NodeProps + Default;
    type Events: NodeEvents;

    fn handle_event(
        _event: &InputEvent,
        _state: &mut Self::State,
        _props: &Self::Props,
        _event_trigger: &mut EventTrigger<'_, Self::Events>,
    ) {
    }
    fn render_element(
        render_context: &mut RenderContext,
        parameters: &RenderParameters,
        state: &Self::State,
        props: &Self::Props,
    );

    fn layout_element(
        layout_context: &mut LayoutContext,
        state: &mut Self::State,
        props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints>;
}

pub enum LayoutConstraints {
    Passthrough,
    FixedLayout { width: Float, height: Float },
    HorizontalLayout {},
}

#[derive(Debug, Reflect)]
pub enum NoEvents {}
impl NodeEvents for NoEvents {}
