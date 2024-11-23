use crate::infrastructure::layout_context::LayoutContext;
use crate::infrastructure::text_edit_state::TextEditState;
use crate::nodes::elements::kind::{Element, EventTrigger, LayoutConstraints};
use crate::nodes::events::InputEvent;
use crate::nodes::types::{NodeEvents, NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use crate::types::{Point, Rect, Size};
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
        let mut edit_position = state.edit_position.unwrap_or(props.text.len());
        edit_position = edit_position.clamp(0, props.text.len());
        let mut text_edit_state = TextEditState::new(&props.text, edit_position);
        text_edit_state.handle_event(event);
        state.edit_position = Some(text_edit_state.cursor_position);
        if let Some(new_text) = text_edit_state.new_text {
            event_trigger(TextInputEvents::Change {
                new_value: new_text,
            });
        }
    }

    fn render_element(
        render_context: &mut RenderContext,
        parameters: &RenderParameters,
        state: &Self::State,
        props: &Self::Props,
    ) {
        let styling = parameters.styling();
        let stroke_width = 2.0f32;
        render_context.add_command(RenderCommand::SetStrokeColor(styling.border_color));
        render_context.add_command(RenderCommand::SetFillColor(styling.background_color));
        render_context.add_command(RenderCommand::SetStrokeWidth(2.0));
        render_context.add_command(RenderCommand::FillRoundRect {
            rect: Rect::new(
                Point::new(stroke_width, stroke_width),
                Size::new(1000.0 - stroke_width * 2.0, 40.0 - stroke_width * 2.0),
            ),
            radius: 2.0,
        });

        render_context.add_command(RenderCommand::SetStrokeColor(styling.text_color));
        if let Some(mut edit_position) = state.edit_position {
            edit_position = edit_position.clamp(0, props.text.len());
            if render_context.time() % 1.0 < 0.5 {
                let size = render_context
                    .measure_text(&props.text[0..edit_position])
                    .unwrap();
                render_context.add_command(RenderCommand::FillRect {
                    rect: Rect::new(
                        Point::new(11.0 + size.width, stroke_width + 2.0),
                        Size::new(2.0, 30.0),
                    ),
                });
            }
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
    pub edit_position: Option<usize>,
}

impl NodeState for TextInputElementState {}

#[derive(Reflect, Debug)]
pub enum TextInputEvents {
    Change { new_value: String },
}
impl NodeEvents for TextInputEvents {}
