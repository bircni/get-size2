#![doc = include_str!("./lib.md")]
// `doc_cfg` enables the automatic feature badges on docs.rs, so every `std`, `alloc` and
// optional dependency gated item shows what it requires.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
// Not part of the workspace lints, since those apply to the examples as well, which would then
// have to document every item they declare.
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "derive")]
pub use get_size_derive2::*;

mod impls;
mod tracker;
pub use tracker::*;

/// Names the generated implementations rely on, whatever the scope of the deriving type holds.
///
/// Not part of the public API: the derive macro of the same version is its only user.
#[doc(hidden)]
pub mod __private {
    pub use core::primitive::usize;
}
// The test suite exercises the `std` implementations and is therefore only
// compiled when the `std` feature is active.
#[cfg(all(test, feature = "std"))]
mod tests;

/// Determines how many bytes the object occupies inside the heap.
pub fn heap_size<T: GetSize>(value: &T) -> usize {
    value.get_heap_size()
}

/// The tracker used by [`GetSize::get_heap_size`] and [`GetSize::get_size`].
///
/// This is [`StandardTracker`] if the `alloc` feature is active, since tracking requires an
/// allocated set of already seen addresses. Without `alloc` there is no shared ownership type
/// (`Rc`/`Arc`) to track, so [`NoTracker`] is used instead.
#[cfg(feature = "alloc")]
pub type DefaultTracker = StandardTracker;

/// The tracker used by [`GetSize::get_heap_size`] and [`GetSize::get_size`].
///
/// This is [`StandardTracker`] if the `alloc` feature is active, since tracking requires an
/// allocated set of already seen addresses. Without `alloc` there is no shared ownership type
/// (`Rc`/`Arc`) to track, so [`NoTracker`] is used instead.
#[cfg(not(feature = "alloc"))]
pub type DefaultTracker = NoTracker;

/// Creates a fresh [`DefaultTracker`].
///
/// An implementation detail of [`GetSize::get_heap_size`], and of the implementations generated
/// by earlier versions of the derive macro. Create the tracker you need directly instead.
#[cfg(feature = "alloc")]
#[doc(hidden)]
#[must_use]
pub fn default_tracker() -> DefaultTracker {
    StandardTracker::new()
}

/// Creates a fresh [`DefaultTracker`].
///
/// An implementation detail of [`GetSize::get_heap_size`], and of the implementations generated
/// by earlier versions of the derive macro. Create the tracker you need directly instead.
#[cfg(not(feature = "alloc"))]
#[doc(hidden)]
#[must_use]
pub const fn default_tracker() -> DefaultTracker {
    NoTracker::new(true)
}

/// Determine the size in bytes an object occupies inside RAM.
pub trait GetSize: Sized {
    /// Determines how may bytes this object occupies inside the stack.
    ///
    /// The default implementation uses [`core::mem::size_of`] and should work for almost all types.
    #[must_use]
    fn get_stack_size() -> usize {
        core::mem::size_of::<Self>()
    }

    /// Determines how many bytes this object occupies inside the heap.
    ///
    /// The default implementation simply delegates to [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker)
    /// with a fresh [`DefaultTracker`], so an allocation which is shared through several `Rc` or
    /// `Arc` handles inside this object is only accounted for once. Pass a [`NoTracker`] to
    /// [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker) to count every handle.
    ///
    /// This method is not meant to be implemented directly, and only exists for convenience.
    fn get_heap_size(&self) -> usize {
        let tracker = default_tracker();
        Self::get_heap_size_with_tracker(self, tracker).0
    }

    /// Determines how many bytes this object occupies inside the heap while using a `tracker`.
    ///
    /// The default implementation returns 0, assuming the object is fully allocated on the stack.
    /// It must be adjusted as appropriate for objects which hold data inside the heap.
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (0, tracker)
    }

    /// Determines the total size of the object.
    ///
    /// The default implementation simply adds up the results of [`get_stack_size`](Self::get_stack_size)
    /// and [`get_heap_size`](Self::get_heap_size) and is not meant to be changed.
    fn get_size(&self) -> usize {
        Self::get_stack_size() + GetSize::get_heap_size(self)
    }

    /// Determines the total size of the object while using a `tracker`.
    ///
    /// The default implementation simply adds up the results of [`get_stack_size`](Self::get_stack_size)
    /// and [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker) and is not meant to
    /// be changed.
    fn get_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        let stack_size = Self::get_stack_size();
        let (heap_size, tracker) = Self::get_heap_size_with_tracker(self, tracker);
        (stack_size + heap_size, tracker)
    }
}
