use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::rc::Rc;
use alloc::string::String;
// `Arc` is only provided by `alloc` on targets with pointer sized atomics.
#[cfg(target_has_atomic = "ptr")]
use alloc::sync::Arc;

use crate::{GetSize, GetSizeTracker};

impl GetSize for String {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.capacity(), tracker)
    }
}

impl GetSize for CString {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.as_bytes_with_nul().len(), tracker)
    }
}

impl GetSize for Box<str> {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl GetSize for Rc<str> {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, mut tracker: T) -> (usize, T) {
        if !tracker.track(Self::as_ptr(self).cast::<()>()) {
            return (0, tracker);
        }

        (self.len(), tracker)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl GetSize for Arc<str> {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, mut tracker: T) -> (usize, T) {
        if !tracker.track(Self::as_ptr(self).cast::<()>()) {
            return (0, tracker);
        }

        (self.len(), tracker)
    }
}
