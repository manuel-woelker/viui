use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::elements::kind::{Element, EventTrigger, LayoutConstraints};
use crate::nodes::events::{InputEvent, InputEventKind};
use crate::nodes::types::{NodeEvents, NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use crate::types::{Point, Rect, Size};
use bevy_reflect::Reflect;

pub struct ButtonElement {}

impl Element for ButtonElement {
    const NAME: &'static str = "button";
    type State = ButtonElementState;
    type Props = ButtonElementProps;
    type Events = ButtonEvents;

    fn handle_event(
        event: &InputEvent,
        state: &mut Self::State,
        _props: &Self::Props,
        event_trigger: &mut EventTrigger<ButtonEvents>,
    ) {
        match event.kind() {
            InputEventKind::MouseOver => {
                state.is_hovering = true;
            }
            InputEventKind::MouseOut => {
                state.is_hovering = false;
            }
            InputEventKind::MousePress(..) => {
                state.is_pressed = true;
                event_trigger(ButtonEvents::Click);
            }
            InputEventKind::MouseRelease(..) => {
                state.is_pressed = false;
            }
            _ => {}
        }
    }

    fn render_element(
        render_context: &mut RenderContext,
        parameters: &RenderParameters,
        state: &Self::State,
        props: &Self::Props,
    ) {
        let styling = parameters.styling();
        if state.is_pressed {
            render_context.add_command(RenderCommand::SetFillColor(styling.button_pressed_color));
        } else if state.is_hovering {
            render_context.add_command(RenderCommand::SetFillColor(styling.button_hover_color));
        } else {
            render_context.add_command(RenderCommand::SetFillColor(styling.button_color));
        }
        let stroke_width = 2.0f32;
        render_context.add_command(RenderCommand::SetStrokeColor(styling.text_color));
        render_context.add_command(RenderCommand::SetStrokeWidth(2.0));
        render_context.add_command(RenderCommand::FillRoundRect {
            rect: Rect::new(
                Point::new(stroke_width, stroke_width),
                Size::new(200.0 - stroke_width * 2.0, 40.0 - stroke_width * 2.0),
            ),
            radius: 5.0,
        });
        render_context.add_command(RenderCommand::Translate { x: 15.0, y: 25.0 });
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
pub struct ButtonElementProps {
    pub label: String,
}

impl NodeProps for ButtonElementProps {}

#[derive(Reflect, Debug, Default)]
pub struct ButtonElementState {
    pub is_hovering: bool,
    pub is_pressed: bool,
}
impl NodeState for ButtonElementState {}

#[derive(Reflect, Debug)]
pub enum ButtonEvents {
    Click,
}
impl NodeEvents for ButtonEvents {}
