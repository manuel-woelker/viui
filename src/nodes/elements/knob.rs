use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::elements::kind::{Element, EventTrigger, LayoutConstraints};
use crate::nodes::events::{InputEvent, InputEventKind};
use crate::nodes::types::{NodeEvents, NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use crate::types::{Float, Point, Rect, Size};
use bevy_reflect::Reflect;
use std::f32::consts::PI;

pub struct KnobElement {}

impl Element for KnobElement {
    const NAME: &'static str = "knob";
    type State = KnobElementState;
    type Props = KnobElementProps;
    type Events = KnobEvents;

    fn handle_event(
        event: &InputEvent,
        state: &mut Self::State,
        props: &Self::Props,
        event_trigger: &mut EventTrigger<KnobEvents>,
    ) {
        match event.kind() {
            InputEventKind::MouseOver => {
                state.is_hovering = true;
            }
            InputEventKind::MouseOut => {
                state.is_hovering = false;
            }
            InputEventKind::MousePress(position) => {
                state.drag_start_x = position.x;
                state.drag_start_y = position.y;
                state.drag_start_value = props.value;
                state.is_pressed = true;
                state.is_dragging = true;
            }
            InputEventKind::MouseRelease(..) => {
                state.is_pressed = false;
                state.is_dragging = false;
            }
            InputEventKind::MouseMove(position) => {
                if state.is_dragging {
                    let delta_x = position.x - state.drag_start_x;
                    let delta_y = position.y - state.drag_start_y;
                    let delta = (delta_x - delta_y) / 10.0;
                    let new_value = state.drag_start_value + delta;
                    let new_value = new_value.clamp(props.min_value, props.max_value);
                    event_trigger(KnobEvents::Change { new_value });
                }
            }
            _ => (),
        }
    }

    fn render_element(
        render_context: &mut RenderContext,
        parameters: &RenderParameters,
        _state: &Self::State,
        props: &Self::Props,
    ) {
        let styling = parameters.styling();
        // clamp value to min and max
        let value = props.value.clamp(props.min_value, props.max_value);
        let relative_value = (value - props.min_value) / (props.max_value - props.min_value);
        let min_angle = 0.75 * PI;
        let max_angle = 2.25 * PI;
        let angle = min_angle + relative_value * (max_angle - min_angle);
        render_context.add_command(RenderCommand::SetStrokeWidth(2.0));
        render_context.add_command(RenderCommand::SetStrokeColor(styling.border_color));
        render_context.add_command(RenderCommand::SetFillColor(styling.button_color));
        render_context.add_command(RenderCommand::Translate { x: 10.0, y: 20.0 });
        render_context.add_command(RenderCommand::FillRoundRect {
            rect: Rect::new(Point::new(10.0, 10.0), Size::new(40.0, 40.0)),
            radius: 20.0,
        });
        let center_x = 30.0;
        let center_y = 30.0;
        let radius = 20.0;
        let start_x = center_x + radius * angle.cos();
        let start_y = center_y + radius * angle.sin();
        let end_x = center_x + (radius - 10.0) * angle.cos();
        let end_y = center_y + (radius - 10.0) * angle.sin();

        render_context.add_command(RenderCommand::Line {
            start: Point::new(start_x, start_y),
            end: Point::new(end_x, end_y),
        });
        render_context.add_command(RenderCommand::SetStrokeColor(styling.inactive_color));
        render_context.add_command(RenderCommand::SetStrokeWidth(3.0));
        render_context.add_command(RenderCommand::Arc {
            center: Point::new(30.0, 30.0),
            radius: 30.0,
            start_angle: min_angle,
            end_angle: max_angle,
        });
        render_context.add_command(RenderCommand::SetStrokeColor(styling.highlight_color));
        render_context.add_command(RenderCommand::Arc {
            center: Point::new(30.0, 30.0),
            radius: 30.0,
            start_angle: min_angle,
            end_angle: angle,
        });

        render_context.add_command(RenderCommand::SetStrokeColor(styling.text_color));
        render_context.add_command(RenderCommand::Translate { x: 10.0, y: 80.0 });
        render_context.add_command(RenderCommand::DrawText(props.label.clone()));
    }

    fn layout_element(
        _layout_context: &mut LayoutContext,
        _state: &mut Self::State,
        _props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::FixedLayout {
            width: 200.0,
            height: 140.0,
        })
    }
}

#[derive(Default, Reflect, Debug)]
pub struct KnobElementProps {
    pub min_value: f32,
    pub max_value: f32,
    pub value: f32,
    pub label: String,
}

impl NodeProps for KnobElementProps {}

#[derive(Reflect, Debug, Default)]
pub struct KnobElementState {
    pub is_hovering: bool,
    pub is_pressed: bool,
    pub is_dragging: bool,
    pub drag_start_x: Float,
    pub drag_start_y: Float,
    pub drag_start_value: Float,
}

impl NodeState for KnobElementState {}

#[derive(Reflect, Debug)]
pub enum KnobEvents {
    Change { new_value: Float },
}
impl NodeEvents for KnobEvents {}
