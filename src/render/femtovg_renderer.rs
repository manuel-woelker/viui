use crate::render::command::RenderCommand;
use crate::render::renderer::CommandRenderer;
use femtovg::{Baseline, Canvas, Color, Paint, Path};

pub struct FemtovgRenderer<'a, T: femtovg::Renderer> {
    canvas: &'a mut Canvas<T>,
}

impl <'a, T: femtovg::Renderer> FemtovgRenderer<'a, T> {
    pub fn new(canvas: &'a mut Canvas<T>) -> Self {
        Self {
            canvas,
        }
    }
}

impl <'a, T: femtovg::Renderer> CommandRenderer for FemtovgRenderer<'a, T> {
    fn render(&mut self, commands: &[RenderCommand]) {
        let mut fill_paint = Paint::color(Color::hsl(0.0, 0.0, 1.0)).with_text_baseline(Baseline::Middle).with_font_size(20.0).with_anti_alias(true);
        let mut stroke_paint = Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_text_baseline(Baseline::Middle).with_font_size(20.0).with_anti_alias(true);
        let canvas = &mut self.canvas;
        for command in commands {
            match command {
                RenderCommand::FillRect { .. } => {}
                RenderCommand::FillRoundRect { rect, radius } => {
                    let mut path = Path::new();
                    path.rounded_rect(
                        0.0,
                        0.0,
                        200.0,
                        40.0,
                        *radius,
                    );

                    canvas.fill_path(&path, &fill_paint);
                    canvas.stroke_path(&path, &stroke_paint);
                }
                RenderCommand::Translate { x, y } => {
                    canvas.translate(*x, *y);
                }
                RenderCommand::ResetTransform => {
                    canvas.reset_transform();
                }
                RenderCommand::DrawText(text   ) => {
                    canvas.fill_text(0.0, 0.0, text, &stroke_paint).unwrap();
                }
                RenderCommand::Save => {
                    canvas.save();
                }
                RenderCommand::Restore => {
                    canvas.restore();
                }
                RenderCommand::SetFillColor(color) => {
                    fill_paint.set_color(Color::rgba(color.r, color.g, color.b, color.a));
                }
                RenderCommand::SetStrokeColor(color) => {
                    stroke_paint.set_color(Color::rgba(color.r, color.g, color.b, color.a));
                }
            }
        }
    }
}