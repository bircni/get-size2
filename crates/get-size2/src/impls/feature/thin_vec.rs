use crate::{GetSize, GetSizeTracker};

// The capacity and the length are stored in front of the elements.
const HEADER_SIZE: usize = 2 * size_of::<usize>();

impl<T> GetSize for thin_vec::ThinVec<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // A vector which never held an element points at a shared static header, and one of
        // zero sized elements never allocates.
        if !self.has_capacity() || size_of::<T>() == 0 {
            return (0, tracker);
        }

        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        // The header is padded up to the alignment of the elements.
        let header_size = HEADER_SIZE.max(align_of::<T>());
        let allocation_size = self.capacity() * T::get_stack_size();
        (size + header_size + allocation_size, tracker)
    }
}
