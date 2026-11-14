use crate::{GetSize, GetSizeTracker};

impl GetSize for bytes::Bytes {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, mut tracker: T) -> (usize, T) {
        // Clones share their buffer, so it is tracked like any other shared allocation. Whether
        // a `Bytes` was created from a `'static` slice cannot be told through the public API,
        // such a buffer is therefore counted as well.
        if self.is_empty() || !tracker.track(self.as_ptr()) {
            return (0, tracker);
        }

        (self.len(), tracker)
    }
}

impl GetSize for bytes::BytesMut {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.capacity(), tracker)
    }
}
