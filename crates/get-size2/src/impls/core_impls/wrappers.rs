use core::cell::RefCell;

use crate::{GetSize, GetSizeTracker};

impl<T> GetSize for Option<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        match self {
            None => (0, tracker),
            Some(value) => T::get_heap_size_with_tracker(value, tracker),
        }
    }
}

impl<T, E> GetSize for Result<T, E>
where
    T: GetSize,
    E: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // The result's stack size already accounts for the values stack size.
        match self {
            Ok(value) => T::get_heap_size_with_tracker(value, tracker),
            Err(err) => E::get_heap_size_with_tracker(err, tracker),
        }
    }
}

impl<T> GetSize for RefCell<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `RefCell` holds its data on the stack.
        // Use try_borrow to avoid panicking if the RefCell is already mutably borrowed
        match self.try_borrow() {
            Ok(borrowed) => T::get_heap_size_with_tracker(&*borrowed, tracker),
            Err(_) => {
                // If the RefCell is already mutably borrowed, we cannot safely access it.
                // Return 0 for heap size to avoid panic, though this is a rare edge case.
                (0, tracker)
            }
        }
    }
}
