use orx_concurrent_vec::{ConcurrentElement, ConcurrentVec, IntoConcurrentPinnedVec};

use crate::{GetSize, GetSizeTracker};

// `ConcurrentVec::new()` eagerly allocates an initial fragment, so
// `capacity()` is nonzero even for an empty vec. Backing `SplitVec`
// fragment headers are not counted (bounded by a small constant for
// the default Doubling growth strategy).

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
