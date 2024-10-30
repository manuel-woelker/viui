use std::marker::PhantomData;
use bevy_reflect::{GetPath, ParsedPath, Reflect};

pub struct ObservableState {
    value: Box<dyn Reflect>,
    changes: Vec<Change>,
}

impl ObservableState {
    pub fn new<T: Reflect>(value: T) -> Self {
        Self { value: Box::new(value), changes: Vec::new() }
    }

    pub fn state(&self) -> &dyn Reflect {
        &*self.value
    }

    pub fn inspect<T: Reflect>(&self, path: TypedPath<T>) -> &T {
        self.value.path(&path.path).unwrap()
    }


    pub fn apply_change(&mut self, label: impl Into<String>, mutation: impl Fn(&mut Mutator)) {
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

pub struct Mutator<'a> {
    label: String,
    state: &'a mut ObservableState,
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

impl <'a> Mutator<'a> {
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
    use bevy_reflect::{GetPath, ParsedPath, Reflect};
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
        assert_eq!(19, *state.state().path("counter").unwrap());
        state.apply_change("Increment counter", |mutator| {
            mutator.mutate(counter_path,|counter| *counter+=1);
        });
        assert_eq!(20, *state.state().path("counter").unwrap());
        dbg!(&state.changes);
        state.undo();
        assert_eq!(19, *state.state().path("counter").unwrap());
    }
}
