use crate::render::command::RenderCommand;

pub trait CommandRenderer {
    fn render(&mut self, commands: &[RenderCommand]);
}