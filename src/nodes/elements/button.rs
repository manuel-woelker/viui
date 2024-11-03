use crate::nodes::elements::kind::Element;
use crate::nodes::events::{NodeEvent, NodeEventKind};
use crate::nodes::types::{NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::types::{Color, Point, Rect, Size};
use bevy_reflect::Reflect;

pub struct ButtonElement {}

impl Element for ButtonElement {
    const NAME: &'static str = "button";
    type State = ButtonElementState;
    type Props = ButtonElementProps;

    fn handle_event(event: &NodeEvent, state: &mut Self::State, _props: &Self::Props) {
        match event.kind() {
            NodeEventKind::MouseOver => {
                state.is_hovering = true;
            }
            NodeEventKind::MouseOut => {
                state.is_hovering = false;
            }
            NodeEventKind::MousePress => {
                state.is_pressed = true;
            }
            NodeEventKind::MouseRelease => {
                state.is_pressed = false;
            }
        }
    }

    fn render_element(
        render_queue: &mut Vec<RenderCommand>,
        state: &Self::State,
        props: &Self::Props,
    ) {
        if state.is_pressed {
            render_queue.push(RenderCommand::SetFillColor(Color::new(250, 250, 250, 255)));
        } else if state.is_hovering {
            render_queue.push(RenderCommand::SetFillColor(Color::new(230, 230, 230, 255)));
        } else {
            render_queue.push(RenderCommand::SetFillColor(Color::new(220, 220, 220, 255)));
        }
        render_queue.push(RenderCommand::FillRoundRect {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 40.0)),
            radius: 5.0,
        });
        render_queue.push(RenderCommand::Translate { x: 10.0, y: 20.0 });
        render_queue.push(RenderCommand::DrawText(props.label.clone()));
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

#[derive(Reflect, Default, Debug)]
pub struct Text {
    pub parts: Vec<TextPart>,
}

#[derive(Reflect, Debug)]
pub enum TextPart {
    FixedText(String),
    VariableText(String),
}
