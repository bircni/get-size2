use std::sync::{Mutex, OnceLock, RwLock, TryLockError};

use crate::{GetSize, GetSizeTracker};

impl<T> GetSize for Mutex<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `Mutex` holds its data on the stack. Measuring must never block, as
        // the lock might be held by the calling thread itself, so a locked `Mutex` reports 0.
        match self.try_lock() {
            Ok(guard) => T::get_heap_size_with_tracker(&*guard, tracker),
            Err(TryLockError::Poisoned(poisoned)) => {
                T::get_heap_size_with_tracker(&*poisoned.into_inner(), tracker)
            }
            Err(TryLockError::WouldBlock) => (0, tracker),
        }
    }
}

impl<T> GetSize for RwLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `RwLock` holds its data on the stack. Measuring must never block, as
        // the lock might be held by the calling thread itself, so a write locked `RwLock`
        // reports 0.
        match self.try_read() {
            Ok(guard) => T::get_heap_size_with_tracker(&*guard, tracker),
            Err(TryLockError::Poisoned(poisoned)) => {
                T::get_heap_size_with_tracker(&*poisoned.into_inner(), tracker)
            }
            Err(TryLockError::WouldBlock) => (0, tracker),
        }
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
