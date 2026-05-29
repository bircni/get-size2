use orx_concurrent_vec::{ConcurrentElement, ConcurrentVec, IntoConcurrentPinnedVec};

use crate::{GetSize, GetSizeTracker};

// `ConcurrentVec<T, P>` is backed by a `PinnedVec` of `ConcurrentElement<T>`
// slots; each slot is a `ConcurrentElement<T>` (an atomic-state wrapper
// around `T`) sized at `size_of::<ConcurrentElement<T>>()`. Heap layout
// matches `Vec`: `capacity()` × per-slot size, plus the recursive heap
// of each occupied element. This mirrors the `Vec<T>` impl pattern in
// `collections.rs`.
//
// Not counted: per-fragment metadata in the backing `SplitVec`
// (fragment pointer/length pairs). For a `Doubling` growth strategy
// this is bounded by ~`log2(capacity)` fragments — tens of bytes total
// regardless of corpus size.

impl<T, P> GetSize for ConcurrentVec<T, P>
where
    T: GetSize,
    P: IntoConcurrentPinnedVec<ConcurrentElement<T>>,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (heap, tracker) = self.iter().fold((0usize, tracker), |(size, tr), elem| {
            let (elem_size, tr) = elem.map(|t: &T| T::get_heap_size_with_tracker(t, tr));
            (size + elem_size, tr)
        });
        let allocation = self.capacity() * core::mem::size_of::<ConcurrentElement<T>>();
        (heap + allocation, tracker)
    }
}
