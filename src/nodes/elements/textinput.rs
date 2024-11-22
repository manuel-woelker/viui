use crate::infrastructure::layout_context::LayoutContext;
use crate::infrastructure::measure_text::TextMeasurer;
use crate::nodes::elements::kind::{Element, EventTrigger, LayoutConstraints};
use crate::nodes::events::{InputEvent, InputEventKind};
use crate::nodes::types::{NodeEvents, NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::result::ViuiResult;
use crate::types::{Color, Point, Rect, Size};
use bevy_reflect::Reflect;

pub struct TextInputElement {}

impl Element for TextInputElement {
    const NAME: &'static str = "textinput";
    type State = TextInputElementState;
    type Props = TextInputElementProps;
    type Events = TextInputEvents;
    fn handle_event(
        event: &InputEvent,
        state: &mut Self::State,
        props: &Self::Props,
        event_trigger: &mut EventTrigger<'_, Self::Events>,
    ) {
        match event.kind() {
            InputEventKind::MousePress { .. } => {
                state.is_editing = true;
            }
            InputEventKind::Character(character) => {
                let mut new_value = props.text.clone();
                if *character == '\u{8}' {
                    new_value.pop();
                } else {
                    new_value.push(*character);
                }
                event_trigger(TextInputEvents::Change { new_value });
            }
            _ => {}
        }
    }

    fn render_element(
        render_context: &mut RenderContext,
        state: &Self::State,
        props: &Self::Props,
    ) {
        let stroke_width = 2.0f32;
        render_context.add_command(RenderCommand::SetStrokeColor(Color::new(0, 0, 0, 255)));
        render_context.add_command(RenderCommand::SetFillColor(Color::new(255, 255, 255, 255)));
        render_context.add_command(RenderCommand::SetStrokeWidth(2.0));
        render_context.add_command(RenderCommand::FillRoundRect {
            rect: Rect::new(
                Point::new(stroke_width, stroke_width),
                Size::new(1000.0 - stroke_width * 2.0, 40.0 - stroke_width * 2.0),
            ),
            radius: 2.0,
        });

        render_context.add_command(RenderCommand::SetFillColor(Color::new(0, 0, 0, 255)));
        if state.is_editing && render_context.time() % 1.0 < 0.5 {
            let size = TextMeasurer::from_resource("assets/fonts/OpenSans-Regular.ttf")
                .unwrap()
                .measure_text(&props.text, 25.0)
                .unwrap();
            render_context.add_command(RenderCommand::FillRect {
                rect: Rect::new(
                    Point::new(12.0 + size.width, stroke_width + 2.0),
                    Size::new(2.0, 30.0),
                ),
            });
        }

        render_context.add_command(RenderCommand::Translate { x: 10.0, y: 25.0 });
        render_context.add_command(RenderCommand::DrawText(props.text.clone()));
    }

    fn layout_element(
        _layout_context: &mut LayoutContext,
        _state: &mut Self::State,
        _props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::FixedLayout {
            width: 1000.0,
            height: 40.0,
        })
    }
}

#[derive(Default, Reflect, Debug)]
pub struct TextInputElementProps {
    pub text: String,
}

impl NodeProps for TextInputElementProps {}

#[derive(Default, Reflect, Debug)]
pub struct TextInputElementState {
    pub is_editing: bool,
}

impl NodeState for TextInputElementState {}

#[derive(Reflect, Debug)]
pub enum TextInputEvents {
    Change { new_value: String },
}
impl NodeEvents for TextInputEvents {}
