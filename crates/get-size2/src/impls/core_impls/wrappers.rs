use core::cell::{Cell, OnceCell, RefCell};
use core::cmp::Reverse;
use core::mem::ManuallyDrop;
use core::num::{Saturating, Wrapping};
use core::ops::Bound;

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

impl<T> GetSize for Cell<T>
where
    T: GetSize + Copy,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // A `Cell` hands out no references, so the value is copied out. A `Copy` type cannot
        // own heap memory itself, but may still account for some by a manual implementation.
        T::get_heap_size_with_tracker(&self.get(), tracker)
    }
}

impl<T> GetSize for OnceCell<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        match self.get() {
            None => (0, tracker),
            Some(value) => T::get_heap_size_with_tracker(value, tracker),
        }
    }
}

impl<T> GetSize for Bound<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        match self {
            Self::Included(value) | Self::Excluded(value) => {
                T::get_heap_size_with_tracker(value, tracker)
            }
            Self::Unbounded => (0, tracker),
        }
    }
}

/// Implements [`GetSize`] for a transparent wrapper by measuring the value it derefs to.
macro_rules! impl_size_transparent {
    ($name:ident, $value:ident => $inner:expr) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
                let $value = self;
                T::get_heap_size_with_tracker($inner, tracker)
            }
        }
    };
}

impl_size_transparent!(Wrapping, value => &value.0);
impl_size_transparent!(Saturating, value => &value.0);
impl_size_transparent!(Reverse, value => &value.0);
impl_size_transparent!(ManuallyDrop, value => &**value);
