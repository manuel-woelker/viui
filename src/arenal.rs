use rand::random;
use std::num::NonZeroU16;
use std::ops::{Index, IndexMut};

pub struct Arenal<T> {
    arenal_id: ArenalId,
    entries: Vec<Entry<T>>,
}

type OffsetType = u32;
type ArenalId = u16;
type Generation = NonZeroU16;

enum Entry<T> {
    Occupied(Occupied<T>),
    #[allow(unused)]
    Empty(Empty),
}

struct Occupied<T> {
    generation: Generation,
    value: T,
}

struct Empty {}

pub struct Idx<T> {
    arenal_id: ArenalId,
    generation: Generation,
    offset: OffsetType,
    marker: std::marker::PhantomData<T>,
}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        Idx {
            arenal_id: self.arenal_id,
            generation: self.generation,
            offset: self.offset,
            marker: std::marker::PhantomData,
        }
    }
}

impl<T> Copy for Idx<T> {}

impl<T> Default for Arenal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arenal<T> {
    pub fn new() -> Self {
        Self {
            arenal_id: random(),
            entries: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.arenal_id = random();
    }

    pub fn insert(&mut self, value: T) -> Idx<T> {
        let generation: Generation = Generation::new(1).unwrap();
        let index = Idx {
            arenal_id: self.arenal_id,
            generation,
            offset: self.entries.len() as u32,
            marker: std::marker::PhantomData,
        };
        self.entries
            .push(Entry::Occupied(Occupied { generation, value }));
        index
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut().filter_map(|item| {
            if let Entry::Occupied(o) = item {
                Some(&mut o.value)
            } else {
                None
            }
        })
    }

    pub fn entries_mut_indexed(&mut self) -> impl Iterator<Item = (&mut T, Idx<T>)> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(offset, item)| {
                if let Entry::Occupied(o) = item {
                    Some((
                        &mut o.value,
                        Idx {
                            arenal_id: self.arenal_id,
                            generation: o.generation,
                            offset: offset as u32,
                            marker: std::marker::PhantomData,
                        },
                    ))
                } else {
                    None
                }
            })
    }

    pub fn entries(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().filter_map(|item| {
            if let Entry::Occupied(o) = item {
                Some(&o.value)
            } else {
                None
            }
        })
    }
}

impl<T> Index<&Idx<T>> for Arenal<T> {
    type Output = T;
    fn index(&self, idx: &Idx<T>) -> &T {
        if idx.arenal_id != self.arenal_id {
            panic!(
                "wrong arenal_id in index: {} != {}",
                idx.arenal_id, self.arenal_id
            );
        }
        let entry = &self.entries[idx.offset as usize];
        let Entry::Occupied(Occupied { value, generation }) = entry else {
            panic!("not occupied");
        };
        if idx.generation != *generation {
            panic!(
                "wrong generation in index: {} != {}",
                idx.generation, generation
            );
        }
        value
    }
}

impl<T> IndexMut<&Idx<T>> for Arenal<T> {
    fn index_mut(&mut self, idx: &Idx<T>) -> &mut T {
        if idx.arenal_id != self.arenal_id {
            panic!(
                "wrong arenal_id in index: {} != {}",
                idx.arenal_id, self.arenal_id
            );
        }
        let entry = &mut self.entries[idx.offset as usize];
        let Entry::Occupied(Occupied { value, generation }) = entry else {
            panic!("not occupied");
        };
        if idx.generation != *generation {
            panic!(
                "wrong generation in index: {} != {}",
                idx.generation, generation
            );
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use crate::arenal::{Arenal, Entry, Idx};

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Idx<()>>(), 8);
        assert_eq!(size_of::<Idx<u32>>(), 8);

        assert_eq!(size_of::<Entry<()>>(), 2);
        assert_eq!(size_of::<Entry<u32>>(), 8);
    }

    #[test]
    fn test_insert() {
        let mut arenal: Arenal<&str> = Arenal::new();
        let foo_idx = arenal.insert("foo");
        assert_eq!(arenal.entries.len(), 1);
        assert_eq!(arenal[&foo_idx], "foo");
    }
}
