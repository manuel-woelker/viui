use crate::types::Point;

#[derive(Debug)]
pub struct InputEvent {
    kind: InputEventKind,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum InputEventKind {
    MouseOver,
    MouseOut,
    MouseMove(Point),
    MousePress(Point),
    MouseRelease(Point),
    Character(char),
    KeyInput(KeyboardKey),
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

    pub fn character(character: char) -> Self {
        Self {
            kind: InputEventKind::Character(character),
        }
    }

    pub fn key_input(keyboard_key: KeyboardKey) -> Self {
        Self {
            kind: InputEventKind::KeyInput(keyboard_key),
        }
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
    CharInput(CharInput),
    KeyInput(KeyInput),
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

#[derive(Debug)]
pub struct CharInput {
    pub character: char,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum KeyboardKey {
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    Delete,
    Backspace,
    Enter,
    Escape,
    Tab,
}

#[derive(Debug)]
pub struct KeyInput {
    pub key: KeyboardKey,
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

    pub fn character_input(character: char) -> Self {
        Self {
            kind: UiEventKind::CharInput(CharInput { character }),
        }
    }

    pub fn key_input(keyboard_key: KeyboardKey) -> Self {
        Self {
            kind: UiEventKind::KeyInput(KeyInput { key: keyboard_key }),
        }
    }
}
