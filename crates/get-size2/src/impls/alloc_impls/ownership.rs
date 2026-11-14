use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::rc::{Rc, Weak as RcWeak};
use core::pin::Pin;
// `Arc` is only provided by `alloc` on targets with pointer sized atomics.
#[cfg(target_has_atomic = "ptr")]
use alloc::sync::{Arc, Weak as ArcWeak};

use crate::impls::layout::size_with_header;
use crate::{GetSize, GetSizeTracker};

// A shared allocation stores the strong and the weak reference counter in front of its value.
type Counters = [usize; 2];

impl<T> GetSize for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        match self {
            Self::Borrowed(_borrowed) => (0, tracker),
            Self::Owned(owned) => <T::Owned>::get_heap_size_with_tracker(owned, tracker),
        }
    }
}

impl<T> GetSize for Box<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        T::get_size_with_tracker(&**self, tracker)
    }
}

impl<T> GetSize for Box<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.len() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl<T> GetSize for Rc<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        if !tracker.track(Self::as_ptr(self)) {
            return (0, tracker);
        }

        let (size, tracker) = T::get_heap_size_with_tracker(&**self, tracker);
        (size + size_with_header::<Counters, T>(&**self), tracker)
    }
}

impl<T> GetSize for Rc<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        if !tracker.track(Self::as_ptr(self)) {
            return (0, tracker);
        }

        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + size_with_header::<Counters, [T]>(&**self), tracker)
    }
}

// A weak handle does not keep the value alive, so it does not own it.
impl<T: ?Sized> GetSize for RcWeak<T> {}

// A pinned owning pointer owns exactly what the pointer itself owns. `Pin` only hands out a
// reference to the pointee, which is all that is needed to measure it.

impl<T> GetSize for Pin<Box<T>>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        T::get_size_with_tracker(self.as_ref().get_ref(), tracker)
    }
}

impl<T> GetSize for Pin<Rc<T>>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        let value = self.as_ref().get_ref();

        // The address of the value is what `Rc::as_ptr` returns, so a pinned and an unpinned
        // handle to the same allocation are deduplicated against each other.
        if !tracker.track(core::ptr::from_ref(value)) {
            return (0, tracker);
        }

        let (size, tracker) = T::get_heap_size_with_tracker(value, tracker);
        (size + size_with_header::<Counters, T>(value), tracker)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T> GetSize for Arc<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        if !tracker.track(Self::as_ptr(self)) {
            return (0, tracker);
        }

        let (size, tracker) = T::get_heap_size_with_tracker(&**self, tracker);
        (size + size_with_header::<Counters, T>(&**self), tracker)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T> GetSize for Arc<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        if !tracker.track(Self::as_ptr(self)) {
            return (0, tracker);
        }

        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + size_with_header::<Counters, [T]>(&**self), tracker)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T: ?Sized> GetSize for ArcWeak<T> {}

#[cfg(target_has_atomic = "ptr")]
impl<T> GetSize for Pin<Arc<T>>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        let value = self.as_ref().get_ref();

        if !tracker.track(core::ptr::from_ref(value)) {
            return (0, tracker);
        }

        let (size, tracker) = T::get_heap_size_with_tracker(value, tracker);
        (size + size_with_header::<Counters, T>(value), tracker)
    }
}
