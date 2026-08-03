use std::sync::{Mutex, OnceLock, RwLock};

use crate::{GetSize, GetSizeTracker};

impl<T> GetSize for Mutex<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `Mutex` holds its data on the stack.
        let guard = self
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        T::get_heap_size_with_tracker(&*guard, tracker)
    }
}

impl<T> GetSize for RwLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `RwLock` holds its data on the stack.
        let guard = self
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        T::get_heap_size_with_tracker(&*guard, tracker)
    }
}

impl<T> GetSize for OnceLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `OnceLock` holds its data on the stack.
        match self.get() {
            None => (0, tracker),
            Some(value) => T::get_heap_size_with_tracker(value, tracker),
        }
    }
}
