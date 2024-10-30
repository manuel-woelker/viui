use thunderdome::{Arena, Index};
use crate::widget_model::WidgetState;

pub struct UI {
    state_arena: Arena<Box<dyn WidgetState>>,
}

impl UI {
    pub fn new() -> UI {
        UI {
            state_arena: Arena::new(),
        }
    }

    pub fn add_state<T: WidgetState>(&mut self, state: T) -> Index {
        self.state_arena.insert(Box::new(state))
    }
}