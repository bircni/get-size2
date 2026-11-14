use crate::impls::layout::index_map_size;
use crate::{GetSize, GetSizeTracker};

impl<K, V, S> GetSize for indexmap::IndexMap<K, V, S>
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

        (size + index_map_size::<K, V>(self.capacity()), tracker)
    }
}

impl<T, S> GetSize for indexmap::IndexSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + index_map_size::<T, ()>(self.capacity()), tracker)
    }
}
