//! Uses the public API the way a downstream crate does.
//!
//! The implementations below spell out the full signature of every trait method, so a change to
//! one of them fails to compile here, which makes a breaking change visible in the diff of this
//! file.

use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use get_size2::{DefaultTracker, GetSize, GetSizeTracker, NoTracker, StandardTracker};

/// A tracker with its own bookkeeping, as a downstream crate may implement it.
#[derive(Default)]
struct RecordingTracker {
    seen: Vec<usize>,
}

impl GetSizeTracker for RecordingTracker {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        let addr = addr.cast::<()>() as usize;

        if self.seen.contains(&addr) {
            false
        } else {
            self.seen.push(addr);
            true
        }
    }
}

/// A type implementing every method of the trait by hand.
struct Manual {
    data: Vec<u8>,
}

impl GetSize for Manual {
    fn get_stack_size() -> usize {
        size_of::<Self>()
    }

    fn get_heap_size(&self) -> usize {
        self.get_heap_size_with_tracker(NoTracker::new(true)).0
    }

    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        self.data.get_heap_size_with_tracker(tracker)
    }

    fn get_size(&self) -> usize {
        Self::get_stack_size() + self.get_heap_size()
    }

    fn get_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        let (heap_size, tracker) = self.get_heap_size_with_tracker(tracker);
        (Self::get_stack_size() + heap_size, tracker)
    }
}

/// A type which only implements the one required method.
struct Minimal(Vec<u8>);

impl GetSize for Minimal {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        self.0.get_heap_size_with_tracker(tracker)
    }
}

/// A type relying on the defaults alone.
struct StackOnly(#[expect(dead_code, reason = "only its size is of interest")] u64);

impl GetSize for StackOnly {}

#[derive(GetSize)]
struct Derived {
    manual: Manual,
    minimal: Minimal,
    stack_only: StackOnly,
    shared: (Rc<str>, Rc<str>),
}

#[test]
fn manual_implementations_compose() {
    let shared: Rc<str> = Rc::from("shared");
    let value = Derived {
        manual: Manual {
            data: Vec::with_capacity(10),
        },
        minimal: Minimal(Vec::with_capacity(20)),
        stack_only: StackOnly(1),
        shared: (Rc::clone(&shared), Rc::clone(&shared)),
    };

    let expected = 10 + 20 + shared.get_heap_size();
    assert_eq!(value.get_heap_size(), expected);
    assert_eq!(get_size2::heap_size(&value), expected);
    assert_eq!(value.get_size(), size_of::<Derived>() + expected);
    assert_eq!(Derived::get_stack_size(), size_of::<Derived>());
    assert_eq!(StackOnly(1).get_size(), size_of::<u64>());
}

#[test]
fn custom_trackers_deduplicate() {
    let shared: Rc<str> = Rc::from("shared");
    let pair = (Rc::clone(&shared), Rc::clone(&shared));
    let once = shared.get_heap_size();

    let (size, tracker) = pair.get_heap_size_with_tracker(RecordingTracker::default());
    assert_eq!(size, once);
    assert_eq!(tracker.seen.len(), 1);

    let (size, tracker) = pair.get_size_with_tracker(tracker);
    assert_eq!(size, size_of::<(Rc<str>, Rc<str>)>());
    assert_eq!(tracker.seen.len(), 1);
}

#[test]
fn provided_trackers_are_interchangeable() {
    fn measure<T: GetSizeTracker>(tracker: T) -> usize {
        let shared = Arc::new(String::from("shared"));
        let pair = (Arc::clone(&shared), shared);
        pair.get_heap_size_with_tracker(tracker).0
    }

    let once = Arc::new(String::from("shared")).get_heap_size();

    assert_eq!(measure(StandardTracker::new()), once);
    assert_eq!(measure(DefaultTracker::default()), once);
    assert_eq!(measure(&mut StandardTracker::new()), once);
    assert_eq!(measure(Box::new(StandardTracker::new())), once);
    assert_eq!(measure(Mutex::new(StandardTracker::new())), once);
    assert_eq!(measure(RwLock::new(StandardTracker::new())), once);
    assert_eq!(measure(Arc::new(Mutex::new(StandardTracker::new()))), once);
    assert_eq!(measure(Arc::new(RwLock::new(StandardTracker::new()))), once);

    // A `NoTracker` answers the same for every address.
    assert_eq!(measure(NoTracker::new(true)), 2 * once);
    assert_eq!(measure(NoTracker::new(false)), 0);

    let mut tracker = NoTracker::default();
    assert!(!tracker.answer());
    tracker.set_answer(true);
    assert!(tracker.answer());

    let mut tracker = StandardTracker::default();
    assert!(tracker.track(&raw const once));
    tracker.clear();
    assert!(tracker.track(&raw const once));
}
