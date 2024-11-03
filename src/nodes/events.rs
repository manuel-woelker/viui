use crate::types::Point;

#[derive(Debug)]
pub struct NodeEvent {
    kind: NodeEventKind,
}

#[derive(Debug)]
pub enum NodeEventKind {
    MouseOver,
    MouseOut,
    MousePress,
    MouseRelease,
}

impl NodeEvent {
    pub fn mouse_over() -> Self {
        Self {
            kind: NodeEventKind::MouseOver,
        }
    }
    pub fn mouse_out() -> Self {
        Self {
            kind: NodeEventKind::MouseOut,
        }
    }
    pub fn mouse_press() -> Self {
        Self {
            kind: NodeEventKind::MousePress,
        }
    }
    pub fn mouse_release() -> Self {
        Self {
            kind: NodeEventKind::MouseRelease,
        }
    }

    pub fn kind(&self) -> &NodeEventKind {
        &self.kind
    }
}

#[derive(Debug)]
pub struct UiEvent {
    #[allow(dead_code)]
    kind: UiEventKind,
}

#[derive(Debug)]
pub enum UiEventKind {
    MouseMoved(Point),
    MouseInput(MouseInput),
}

#[derive(Debug)]
pub struct MouseInput {
    pub mouse_event_kind: MouseEventKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MouseEventKind {
    Pressed,
    Released,
}

impl UiEvent {
    pub fn mouse_move(position: Point) -> Self {
        Self {
            kind: UiEventKind::MouseMoved(position),
        }
    }
    pub fn mouse_input(mouse_event_kind: MouseEventKind) -> Self {
        Self {
            kind: UiEventKind::MouseInput(MouseInput { mouse_event_kind }),
        }
    }
}
