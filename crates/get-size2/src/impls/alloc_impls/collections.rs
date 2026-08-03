use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
use alloc::vec::Vec;

use crate::{GetSize, GetSizeTracker};

macro_rules! impl_size_set {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
                let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), elem| {
                    let (elem_size, tracker) = T::get_heap_size_with_tracker(elem, tracker);
                    (size + elem_size, tracker)
                });

                let allocation_size = self.capacity() * T::get_stack_size();
                (size + allocation_size, tracker)
            }
        }
    };
}

macro_rules! impl_size_set_no_capacity {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
                let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), elem| {
                    // We assume that values are held inside the heap.
                    let (elem_size, tracker) = T::get_size_with_tracker(elem, tracker);
                    (size + elem_size, tracker)
                });

                (size, tracker)
            }
        }
    };
}

impl_size_set_no_capacity!(BTreeSet);
impl_size_set!(BinaryHeap);
impl_size_set_no_capacity!(LinkedList);
impl_size_set!(VecDeque);
impl_size_set!(Vec);

impl<K, V> GetSize for BTreeMap<K, V>
where
    K: GetSize,
    V: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        self.iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            })
    }
}
