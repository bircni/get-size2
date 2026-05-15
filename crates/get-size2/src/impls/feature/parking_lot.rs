use crate::{GetSize, GetSizeTracker};

// parking_lot's Mutex/RwLock are infallible (no poisoning) so these are
// simpler than the std::sync variants in `impls/sync_impls.rs` — no
// `unwrap_or_else(PoisonError::into_inner)` recovery needed.

impl<T> GetSize for parking_lot::Mutex<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let guard = self.lock();
        T::get_heap_size_with_tracker(&*guard, tracker)
    }
}

impl<T> GetSize for parking_lot::RwLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let guard = self.read();
        T::get_heap_size_with_tracker(&*guard, tracker)
    }
}

// Tracker-side mirrors of the std::sync impls in `crate::tracker`.
// Letting a `parking_lot::Mutex<Tracker>` work as a tracker keeps API
// parity with the existing `Mutex<Tracker>` / `RwLock<Tracker>` /
// `Arc<Mutex<Tracker>>` / `Arc<RwLock<Tracker>>` patterns.

impl<T: GetSizeTracker> GetSizeTracker for parking_lot::Mutex<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(self.get_mut(), addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for parking_lot::RwLock<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(self.get_mut(), addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for std::sync::Arc<parking_lot::Mutex<T>> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut guard = self.lock();
        GetSizeTracker::track(&mut *guard, addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for std::sync::Arc<parking_lot::RwLock<T>> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut guard = self.write();
        GetSizeTracker::track(&mut *guard, addr)
    }
}
