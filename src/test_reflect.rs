#[cfg(test)]
mod tests {
    use bevy_reflect::serde::ReflectSerializer;
    use bevy_reflect::{GetPath, ParsedPath, Reflect, Struct, TypeRegistry};

    fn mutate<S: Reflect, T: Reflect>(state: &mut S, path: &str, f: impl Fn(&mut T)) {
        let registry = TypeRegistry::default();
        let t = state.path_mut::<T>(path).unwrap();
        dbg!(path);
        dbg!(ParsedPath::parse(path).unwrap());
        let reflect_serializer = ReflectSerializer::new(t, &registry);
        dbg!(ron::to_string(&reflect_serializer).unwrap());
        f(t);
        let reflect_serializer = ReflectSerializer::new(t, &registry);
        dbg!(ron::to_string(&reflect_serializer).unwrap());
    }

    #[derive(Debug, Reflect)]
    struct AppState {
        counter: i32,
        todos: Vec<String>,
    }
    #[test]
    fn it_works() {
        let mut state = AppState {
            counter: 19,
            todos: vec!["Buy milk".to_string()],
        };
        dbg!(&state);
        dbg!(state.field("counter").unwrap().is::<i32>());
        //        dbg!(state.field("counter").unwrap().get_represented_type_info());
        //        dbg!(state.get_represented_type_info());
        //        dbg!(state.get_field::<i32>("counter"));
        mutate(&mut state, "counter", |counter: &mut i32| *counter += 1);
        mutate(&mut state, "todos", |todos: &mut Vec<String>| {
            todos.push("Walk the dog".to_string())
        });
        mutate(&mut state, "todos[0]", |todo: &mut String| *todo += " now");
        let registry = TypeRegistry::default();
        let reflect_serializer = ReflectSerializer::new(&state, &registry);
        dbg!(ron::to_string(&reflect_serializer).unwrap());
        dbg!(state);
    }
}
