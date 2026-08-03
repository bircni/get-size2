#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::collections::BTreeSet;
#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(feature = "std")]
use std::sync::{Arc, Mutex, RwLock};

/// A tracker which makes sure that shared ownership objects are only accounted for once.
pub trait GetSizeTracker {
    /// Tracks an arbitrary object located at `addr`.
    ///
    /// Returns `true` if the reference, as indexed by the pointed to `addr`, has not yet
    /// been seen by this tracker. Otherwise it returns `false`.
    fn track<A>(&mut self, addr: *const A) -> bool;
}

impl<T: GetSizeTracker> GetSizeTracker for &mut T {
    fn track<A>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(*self, addr)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<T: GetSizeTracker> GetSizeTracker for Box<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(&mut **self, addr)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T: GetSizeTracker> GetSizeTracker for Mutex<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let tracker = self
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T: GetSizeTracker> GetSizeTracker for RwLock<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T: GetSizeTracker> GetSizeTracker for Arc<Mutex<T>> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T: GetSizeTracker> GetSizeTracker for Arc<RwLock<T>> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

// The set of addresses already seen by a `StandardTracker`. A `HashSet` needs the random state
// provided by `std`, so `no_std` builds fall back to the `alloc` only `BTreeSet`.
#[cfg(feature = "std")]
type SeenAddresses = HashSet<usize>;
#[cfg(all(feature = "alloc", not(feature = "std")))]
type SeenAddresses = BTreeSet<usize>;

/// A simple standard tracker which can be used to track shared ownership references.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Debug, Default)]
pub struct StandardTracker {
    inner: SeenAddresses,
}

#[cfg(feature = "alloc")]
impl StandardTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[cfg(feature = "alloc")]
impl GetSizeTracker for StandardTracker {
    fn track<A>(&mut self, addr: *const A) -> bool {
        self.inner.insert(addr.addr())
    }
}

/// A pseudo tracker which does not track anything.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoTracker {
    answer: bool,
}

impl NoTracker {
    /// Creates a new pseudo tracker, which will always return the given `answer`.
    #[must_use]
    pub const fn new(answer: bool) -> Self {
        Self { answer }
    }

    /// Get the answer which will always be returned by this pseudo tracker.
    #[must_use]
    pub const fn answer(&self) -> bool {
        self.answer
    }

    /// Changes the answer which will always be returned by this pseudo tracker.
    pub const fn set_answer(&mut self, answer: bool) {
        self.answer = answer;
    }
}

impl GetSizeTracker for NoTracker {
    fn track<A>(&mut self, _addr: *const A) -> bool {
        self.answer
    }
}
