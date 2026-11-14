use crate::{GetSize, GetSizeTracker};

impl GetSize for bytes::Bytes {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        // Neither the shared buffer nor its origin can be identified through the public API: two
        // views may start at the same address, and a `'static` slice looks like any other. Every
        // handle is therefore accounted for with the bytes it can reach.
        (self.len(), tracker)
    }
}

impl GetSize for bytes::BytesMut {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.capacity(), tracker)
    }
}
