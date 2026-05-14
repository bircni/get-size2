use std::hash::{BuildHasher, Hash};

use crate::{GetSize, GetSizeTracker};

impl<K, V, S> GetSize for dashmap::DashMap<K, V, S>
where
    K: GetSize + Eq + Hash,
    V: GetSize,
    S: BuildHasher + Clone,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), entry| {
            let (k_size, tracker) = K::get_heap_size_with_tracker(entry.key(), tracker);
            let (v_size, tracker) = V::get_heap_size_with_tracker(entry.value(), tracker);
            (size + k_size + v_size, tracker)
        });

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl<T, S> GetSize for dashmap::DashSet<T, S>
where
    T: GetSize + Eq + Hash,
    S: BuildHasher + Clone,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), entry| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(entry.key(), tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.capacity() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}
