use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::rc::{Rc, Weak as RcWeak};
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
        if !tracker.track(Self::as_ptr(self).cast::<()>()) {
            return (0, tracker);
        }

        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + size_with_header::<Counters, [T]>(&**self), tracker)
    }
}

impl<T> GetSize for RcWeak<T> {}

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
        if !tracker.track(Self::as_ptr(self).cast::<()>()) {
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
impl<T> GetSize for ArcWeak<T> {}
