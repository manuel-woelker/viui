use crate::ui::RenderBackendMessage;

pub trait RenderBackend {
    fn handle_message(&mut self, message: RenderBackendMessage);
}