#[cfg(test)]
mod tests {
    use facet::{Facet, OpaqueConst};
    use facet_pretty::FacetPretty;
    use facet_reflect::{Peek, PokeUninit};

    #[derive(Debug, PartialEq, Eq, Facet)]
    struct FooBar {
        foo: u64,
        bar: String,
    }

    #[test]
    fn test() {
        let foo_bar = FooBar {
            foo: 42,
            bar: "baz".to_string(),
        };
        assert_eq!(foo_bar.foo, 42);
        assert_eq!(foo_bar.bar, "baz");
        println!("Pretty: {}", foo_bar.pretty());
        let peek = Peek::new(&foo_bar);
        println!("Reflect: {}", peek);
        dbg!(peek.shape());
        dbg!(peek);
        if let Peek::Struct(peek_struct) = peek {
            dbg!(peek_struct.field_count());

            dbg!(peek_struct.field_name(0));
            dbg!(peek_struct.field_value(0));

            dbg!(peek_struct.field_name(1));
            dbg!(peek_struct.field_value(1));

            dbg!(peek_struct.field_name(2));
            dbg!(peek_struct.field_value(2));
            let bar = dbg!(peek_struct.field_value(1)).unwrap();
            dbg!(bar.shape());
            dbg!(bar);
        }

        let (poke, guard) = PokeUninit::alloc::<FooBar>();
        let mut poke = poke.into_struct();
        let bar = String::from("Hello, World!");
        unsafe {
            poke.unchecked_set(0, OpaqueConst::new(&99u64));
            poke.unchecked_set(1, OpaqueConst::new(&bar));
        }
        core::mem::forget(bar);
        let foo_bar = poke.build::<FooBar>(Some(guard));
        dbg!(foo_bar);
    }
}
