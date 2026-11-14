use core::alloc::Layout;
use std::collections::{HashMap, HashSet};

use crate::impls::layout::hash_table_size;
use crate::{GetSize, GetSizeTracker};

impl<K, V, S> GetSize for HashMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        let allocation_size = hash_table_size(self.capacity(), Layout::new::<(K, V)>());
        (size + allocation_size, tracker)
    }
}

impl<T, S> GetSize for HashSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), elem| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(elem, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = hash_table_size(self.capacity(), Layout::new::<T>());
        (size + allocation_size, tracker)
    }
}
