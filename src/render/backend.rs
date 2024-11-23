use crate::nodes::events::UiEvent;
use crate::types::Size;
use crate::ui::RenderBackendMessage;
use crossbeam_channel::{Receiver, Sender};

pub type BackendIndex = usize;

pub struct RenderBackendParameters {
    pub message_receiver: Receiver<RenderBackendMessage>,
    pub backend_index: BackendIndex,
    pub event_sender: Sender<UiEvent>,
    pub initial_window_size: Size,
}
