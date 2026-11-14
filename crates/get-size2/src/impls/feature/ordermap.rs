use core::alloc::Layout;

use crate::impls::layout::hash_table_size;
use crate::{GetSize, GetSizeTracker};

// The entries live in a `Vec`, each next to its cached hash. A separate hash table maps the
// hashes to their position inside that `Vec`.
fn allocation_size<Entry>(capacity: usize) -> usize {
    capacity * size_of::<(usize, Entry)>() + hash_table_size(capacity, Layout::new::<usize>())
}

impl<K, V, S> GetSize for ordermap::OrderMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
    S: core::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        (size + allocation_size::<(K, V)>(self.capacity()), tracker)
    }
}

impl<T, S> GetSize for ordermap::OrderSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + allocation_size::<T>(self.capacity()), tracker)
    }
}
