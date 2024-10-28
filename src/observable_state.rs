use std::marker::PhantomData;
use bevy_reflect::{GetPath, ParsedPath, Reflect};

pub struct ObservableState<T: Reflect> {
    value: T,
    changes: Vec<Change>,
}

impl <T: Reflect> ObservableState<T> {
    pub fn new(value: T) -> Self {
        Self { value, changes: Vec::new() }
    }

    pub fn state(&self) -> &T {
        &self.value
    }

    pub fn apply_change(&mut self, label: impl Into<String>, mutation: impl Fn(&mut Mutator<T>)) {
        let mut mutator = Mutator {
            label: label.into(),
            state: self
        };
        mutation(&mut mutator);
    }

    pub fn undo(&mut self) {
        let change = self.changes.pop().unwrap();
        let t = self.value.reflect_path_mut(&*change.path).unwrap();
        t.set(change.old_value).unwrap();
    }
}

pub struct Mutator<'a, T: Reflect> {
    label: String,
    state: &'a mut ObservableState<T>,
}

pub struct TypedPath<T: Reflect> {
    path: ParsedPath,
    marker: PhantomData<T>,
}
impl <T: Reflect> TypedPath<T> {
    pub fn new(path: ParsedPath) -> Self {
        Self { path, marker: PhantomData }
    }
}

impl <'a, T: Reflect> Mutator<'a, T> {
    pub fn mutate<V: Reflect>(&mut self, path: &TypedPath<V>, f: impl Fn(&mut V)) {
        let t = self.state.value.path_mut::<V>(&path.path).unwrap();
        let old_value = t.clone_value();
        f(t);
        let new_value = t.clone_value();
        self.state.changes.push(Change {
            label: self.label.clone(),
            path: path.path.to_string(),
            old_value,
            new_value,
        })
    }
}

#[derive(Debug)]
pub struct Change {
    label: String,
    path: String,
    old_value: Box<dyn Reflect>,
    new_value: Box<dyn Reflect>,
}

#[cfg(test)]
mod tests {
    use bevy_reflect::{ParsedPath, Reflect};
    use crate::observable_state::{ObservableState, TypedPath};

    #[derive(Debug, Reflect)]
    struct AppState {
        counter: i32,
        todos: Vec<String>,
    }
    #[test]
    fn test_mutate() {
        let mut state = ObservableState::new(AppState {
            counter: 19,
            todos: vec!["Buy milk".to_string()],
        });
        let counter_path = &TypedPath::<i32>::new(ParsedPath::parse("counter").unwrap());
        assert_eq!(19, state.state().counter);
        state.apply_change("Increment counter", |mutator| {
            mutator.mutate(counter_path,|counter| *counter+=1);
        });
        assert_eq!(20, state.state().counter);
        dbg!(&state.changes);
        state.undo();
        assert_eq!(19, state.state().counter);
    }
}
