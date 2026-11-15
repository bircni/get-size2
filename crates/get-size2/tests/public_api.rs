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

/// Helpers named like the local variables of the generated code.
const fn total(value: &[u8]) -> usize {
    value.len()
}

const fn tracker(value: &[u8]) -> usize {
    value.len()
}

const fn field_size(value: &[u8]) -> usize {
    value.len()
}

const fn field_0(value: &[u8]) -> usize {
    value.len()
}

/// A helper named exactly like a local variable of the generated code, prefix included. Only
/// the hygiene of that local keeps the two apart.
const fn __get_size_total(value: &[u8]) -> usize {
    value.len()
}

const fn __get_size_field_0(value: &[u8]) -> usize {
    value.len()
}

#[test]
fn generated_locals_do_not_capture_user_names() {
    #[derive(GetSize)]
    struct Fields {
        total: String,
        tracker: String,
        field_size: String,
        #[get_size(size_fn = total)]
        first: Vec<u8>,
        #[get_size(size_fn = tracker)]
        second: Vec<u8>,
        #[get_size(size_fn = field_size)]
        third: Vec<u8>,
    }

    #[derive(GetSize)]
    enum Variants {
        Named {
            total: String,
            tracker: String,
            field_0: String,
            #[get_size(size_fn = field_0)]
            helped: Vec<u8>,
        },
        Tuple(String, #[get_size(size_fn = total)] Vec<u8>),
        Prefixed {
            __get_size_total: String,
            __get_size_tracker: String,
            #[get_size(size_fn = __get_size_field_0)]
            first: Vec<u8>,
            #[get_size(size_fn = __get_size_total)]
            second: Vec<u8>,
        },
    }

    let fields = Fields {
        total: String::from("a"),
        tracker: String::from("bc"),
        field_size: String::from("def"),
        first: vec![0; 10],
        second: vec![0; 20],
        third: vec![0; 30],
    };
    assert_eq!(fields.get_heap_size(), 1 + 2 + 3 + 10 + 20 + 30);

    let named = Variants::Named {
        total: String::from("a"),
        tracker: String::from("bc"),
        field_0: String::from("def"),
        helped: vec![0; 10],
    };
    assert_eq!(named.get_heap_size(), 1 + 2 + 3 + 10);

    let tuple = Variants::Tuple(String::from("abcd"), vec![0; 10]);
    assert_eq!(tuple.get_heap_size(), 4 + 10);

    let prefixed = Variants::Prefixed {
        __get_size_total: String::from("a"),
        __get_size_tracker: String::from("bc"),
        first: vec![0; 10],
        second: vec![0; 20],
    };
    assert_eq!(prefixed.get_heap_size(), 1 + 2 + 10 + 20);
}

/// Items named like the local variables of the generated code, as a glob import of generated
/// bindings may bring into scope. A local binding of the same name would be read as a pattern
/// matching the constant, or be rejected for shadowing the static.
mod items {
    #![expect(
        non_upper_case_globals,
        non_camel_case_types,
        reason = "the names have to match the ones a generated implementation could use"
    )]

    use get_size2::GetSize;

    pub const total: u8 = 5;
    pub const field_0: u8 = 5;
    pub static tracker: u8 = 5;
    pub struct field_size;
    pub struct usize;

    #[derive(GetSize)]
    pub struct Struct<const total: bool> {
        pub name: String,
        pub data: Vec<u8>,
    }

    // The second field of the tuple variant is bound to a local named after its position.
    #[derive(GetSize)]
    pub enum Enum<const field_1: bool> {
        Tuple(String, Vec<u8>),
        Named { name: String },
    }
}

#[test]
fn generated_locals_do_not_clash_with_user_items() {
    assert_eq!(items::total + items::field_0 + items::tracker, 15);
    let _ = (items::field_size, items::usize);

    let value = items::Struct::<true> {
        name: String::from("name"),
        data: vec![0; 10],
    };
    assert_eq!(value.get_heap_size(), 4 + 10);

    let tuple = items::Enum::<true>::Tuple(String::from("name"), vec![0; 10]);
    assert_eq!(tuple.get_heap_size(), 4 + 10);

    let named = items::Enum::<true>::Named {
        name: String::from("name"),
    };
    assert_eq!(named.get_heap_size(), 4);
}
