use crate::types::Point;

#[derive(Debug)]
pub struct InputEvent {
    kind: InputEventKind,
}

#[derive(Debug)]
pub enum InputEventKind {
    MouseOver,
    MouseOut,
    MouseMove(Point),
    MousePress(Point),
    MouseRelease(Point),
}

impl InputEvent {
    pub fn mouse_over() -> Self {
        Self {
            kind: InputEventKind::MouseOver,
        }
    }
    pub fn mouse_out() -> Self {
        Self {
            kind: InputEventKind::MouseOut,
        }
    }
    pub fn mouse_press(position: Point) -> Self {
        Self {
            kind: InputEventKind::MousePress(position),
        }
    }
    pub fn mouse_release(position: Point) -> Self {
        Self {
            kind: InputEventKind::MouseRelease(position),
        }
    }

    pub fn mouse_move(position: Point) -> Self {
        Self {
            kind: InputEventKind::MouseMove(position),
        }
    }

    pub fn kind(&self) -> &InputEventKind {
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
