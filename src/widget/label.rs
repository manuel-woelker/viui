use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::result::ViuiResult;
use crate::types::{Color, Point, Rect, Size};
use crate::widget::Widget;

pub struct LabelWidget {}

impl Widget for LabelWidget {
    fn render(&self, render_context: &mut RenderContext) -> ViuiResult<()> {
        render_context.add_command(RenderCommand::SetStrokeColor(Color::gray(127)));
        let stroke_width = 2.0f32;
        render_context.add_command(RenderCommand::SetStrokeWidth(stroke_width));
        render_context.add_command(RenderCommand::FillRoundRect {
            rect: Rect::new(
                Point::new(stroke_width, stroke_width),
                Size::new(200.0 - stroke_width * 2.0, 40.0 - stroke_width * 2.0),
            ),
            radius: 5.0,
        });
        render_context.add_command(RenderCommand::SetStrokeColor(Color::gray(127)));
        render_context.add_command(RenderCommand::SetFillColor(Color::gray(200)));
        render_context.add_command(RenderCommand::Translate { x: 25.0, y: 22.0 });
        render_context.add_command(RenderCommand::DrawText("Hello World".into()));
        Ok(())
    }
}
