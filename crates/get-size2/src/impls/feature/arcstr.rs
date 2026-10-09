use crate::{GetSize, GetSizeTracker};

impl GetSize for arcstr::ArcStr {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, mut tracker: T) -> (usize, T) {
        if Self::is_static(self) || !tracker.track(self.as_ptr()) {
            return (0, tracker);
        }

        (self.len(), tracker)
    }
}

impl GetSize for arcstr::Substr {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        // A substring shares ownership of the entire parent allocation.
        self.parent().get_heap_size_with_tracker(tracker)
    }
}
