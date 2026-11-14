use crate::{GetSize, GetSizeTracker};

impl<A: smallvec::Array> GetSize for smallvec::SmallVec<A>
where
    A::Item: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (mut size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = <A::Item>::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        // A vector which spilled to the heap stays there, even once it shrinks below its inline
        // size again.
        if self.spilled() {
            size += self.capacity() * <A::Item>::get_stack_size();
        }

        (size, tracker)
    }
}
