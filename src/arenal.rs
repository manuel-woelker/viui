use std::num::NonZeroU16;
use std::ops::Index;
use rand::random;
use thunderdome::Arena;

pub struct Arenal<T> {
    arenal_id: ArenalId,
    entries: Vec<Entry<T>>,
}

type OffsetType = u32;
type ArenalId = u16;
type Generation = NonZeroU16;

pub enum Entry<T> {
    Occupied(Occupied<T>),
    Empty(Empty),
}

struct Occupied<T> {
    generation: Generation,
    value: T,
}

struct Empty {
}

pub struct Idx<T> {
    arenal_id: ArenalId,
    generation: Generation,
    offset: OffsetType,
    marker: std::marker::PhantomData<T>,
}


impl <T> Arenal<T> {
    pub fn new() -> Self<> {
        Self {
            arenal_id: random(),
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> Idx<T> {
        let generation: Generation = Generation::new(1).unwrap();
        let index = Idx {
            arenal_id: self.arenal_id,
            generation,
            offset: self.entries.len() as u32,
            marker: std::marker::PhantomData,
        };
        self.entries.push(Entry::Occupied(Occupied {
            generation,
            value,
        }));
        index
    }

    pub fn entries(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut().filter_map(|item| if let Entry::Occupied(o) = item { Some(&mut o.value) } else { None })
    }


}

impl <T> Index<&Idx<T>> for Arenal<T> {
    type Output = T;
    fn index(&self, idx: &Idx<T>) -> &T {
        let entry = &self.entries[idx.offset as usize];
        let Entry::Occupied(Occupied { value, generation }) = entry else {
            panic!("not occupied");
        };
        &value
    }
}



#[cfg(test)]
mod tests {
    use bevy_reflect::{ParsedPath, Reflect};
    use crate::arenal::{Arenal, Entry, Idx};
    use crate::observable_state::{ObservableState, TypedPath};

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Idx<()>>(), 8);
        assert_eq!(std::mem::size_of::<Idx<u32>>(), 8);

        assert_eq!(std::mem::size_of::<Entry<()>>(), 2);
        assert_eq!(std::mem::size_of::<Entry<u32>>(), 8);
    }


    #[test]
    fn test_insert() {
        let mut arenal: Arenal<&str> = Arenal::new();
        let foo_idx = arenal.insert("foo");
        assert_eq!(arenal.entries.len(), 1);
        assert_eq!(arenal[foo_idx], "foo");
    }
}
