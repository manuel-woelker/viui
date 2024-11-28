use crate::component::value::ExpressionValue;
use std::collections::HashMap;

pub struct BindingStack {
    stack: Vec<Bindings>,
}

impl BindingStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self) {
        self.stack.push(Bindings::default());
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn add_binding(&mut self, name: String, value: ExpressionValue) {
        self.stack.last_mut().unwrap().bindings.insert(name, value);
    }

    pub fn get_binding(&self, name: &str) -> Option<ExpressionValue> {
        self.stack
            .iter()
            .rev()
            .find_map(|bindings| bindings.bindings.get(name).cloned())
    }
}

#[derive(Default)]
pub struct Bindings {
    bindings: HashMap<String, ExpressionValue>,
}
