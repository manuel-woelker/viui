use crate::types::Point;

#[derive(Debug)]
pub struct NodeEvent {
    kind: NodeEventKind,
}

#[derive(Debug)]
pub enum NodeEventKind {
    MouseOver,
    MouseOut,
    MouseMove(Point),
    MousePress(Point),
    MouseRelease(Point),
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
    pub fn mouse_press(position: Point) -> Self {
        Self {
            kind: NodeEventKind::MousePress(position),
        }
    }
    pub fn mouse_release(position: Point) -> Self {
        Self {
            kind: NodeEventKind::MouseRelease(position),
        }
    }

    pub fn mouse_move(position: Point) -> Self {
        Self {
            kind: NodeEventKind::MouseMove(position),
        }
    }

    pub fn kind(&self) -> &NodeEventKind {
        &self.kind
    }
}

#[derive(Debug)]
pub struct UiEvent {
    pub kind: UiEventKind,
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
