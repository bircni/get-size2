//! Compares the reported heap sizes with what the allocator actually handed out.
//!
//! The implementations for the standard library hash tables and for `indexmap` reconstruct an
//! allocation size which their types do not expose. Measuring the real allocations keeps those
//! reconstructions honest on every target the test suite runs on.

#![expect(
    unsafe_code,
    reason = "a global allocator can only be implemented with unsafe code"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::{CString, OsString};
use std::hash::RandomState;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use get_size2::GetSize;

thread_local! {
    /// The bytes currently allocated by this thread. Counting per thread keeps the tests
    /// independent of each other, and of whatever the test harness allocates meanwhile.
    static LIVE: Cell<isize> = const { Cell::new(0) };
}

/// Forwards to the system allocator while keeping track of the allocated bytes.
struct Counting;

fn record(bytes: isize) {
    // The thread local is gone while a thread shuts down, which no measurement depends on.
    let _ = LIVE.try_with(|live| live.set(live.get() + bytes));
}

fn signed(size: usize) -> isize {
    isize::try_from(size).unwrap_or(isize::MAX)
}

// SAFETY: Every call is forwarded unchanged to the system allocator, which upholds the contract
// of the trait. The bookkeeping in between neither allocates nor touches the returned memory.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(signed(layout.size()));
        // SAFETY: The caller upholds the contract of `alloc`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record(-signed(layout.size()));
        // SAFETY: The caller upholds the contract of `dealloc`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(signed(new_size) - signed(layout.size()));
        // SAFETY: The caller upholds the contract of `realloc`.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Builds a value and returns the bytes it keeps allocated next to the reported heap size.
fn measure<T: GetSize>(build: impl FnOnce() -> T) -> (usize, usize) {
    let before = LIVE.with(Cell::get);
    let value = build();
    let allocated = LIVE.with(Cell::get) - before;
    let reported = value.get_heap_size();

    (usize::try_from(allocated).unwrap_or(usize::MAX), reported)
}

/// Asserts that the reported heap size is exactly what the allocator handed out.
#[track_caller]
fn exact<T: GetSize>(name: &str, build: impl FnOnce() -> T) {
    let (allocated, reported) = measure(build);
    assert_eq!(reported, allocated, "{name}: reported vs. allocated");
}

/// Asserts that the reported heap size is a lower bound which still accounts for most of the
/// allocation, for types whose bookkeeping cannot be measured through their public API.
#[track_caller]
fn lower_bound<T: GetSize>(name: &str, build: impl FnOnce() -> T) {
    let (allocated, reported) = measure(build);
    assert!(
        reported <= allocated,
        "{name}: reported {reported} exceeds the allocated {allocated}"
    );
    assert!(
        reported.saturating_mul(4) >= allocated,
        "{name}: reported {reported} is less than a quarter of the allocated {allocated}"
    );
}

/// A value with an alignment above the one of every header it is stored next to.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(align(64))]
struct Aligned(u64);

impl GetSize for Aligned {}

/// The interpreter is far too slow for the largest cases.
const LARGEST: usize = if cfg!(miri) { 100 } else { 30_000 };

const CAPACITIES: [usize; 12] = [0, 1, 3, 4, 7, 8, 14, 15, 28, 29, 100, LARGEST];

fn keys(count: usize) -> impl Iterator<Item = u64> {
    (0..count).map(|key| key as u64)
}

#[test]
fn owned_buffers_report_their_capacity() {
    exact("Box<u64>", || Box::new(1u64));
    exact("Box<Aligned>", || Box::new(Aligned(1)));
    exact("Box<[u32]>", || vec![1u32; 10].into_boxed_slice());
    exact("Box<str>", || String::from("boxed").into_boxed_str());
    exact("Vec<u8>", || Vec::<u8>::with_capacity(100));
    exact("Vec<Aligned>", || Vec::<Aligned>::with_capacity(3));
    exact("Vec<String>", || {
        vec![String::from("one"), String::from("two")]
    });
    exact("String", || String::with_capacity(33));
    exact("VecDeque<u64>", || VecDeque::<u64>::with_capacity(10));
    exact("BinaryHeap<u64>", || BinaryHeap::<u64>::with_capacity(10));
    exact("CString", || CString::new("c string").unwrap_or_default());
    exact("OsString", || OsString::with_capacity(100));
    exact("PathBuf", || PathBuf::with_capacity(100));
}

#[test]
fn shared_allocations_include_their_counters() {
    exact("Rc<u8>", || Rc::new(1u8));
    exact("Rc<u128>", || Rc::new(1u128));
    exact("Rc<()>", || Rc::new(()));
    exact("Rc<Aligned>", || Rc::new(Aligned(1)));
    exact("Rc<String>", || Rc::new(String::from("shared")));
    exact("Rc<[u8]>", || Rc::<[u8]>::from(vec![1u8; 10]));
    exact("Rc<[u8]> empty", || Rc::<[u8]>::from(Vec::new()));
    exact("Rc<[Aligned]>", || {
        Rc::<[Aligned]>::from(vec![Aligned(1); 3])
    });
    exact("Rc<str>", || Rc::<str>::from("hello"));
    exact("Arc<u8>", || Arc::new(1u8));
    exact("Arc<Aligned>", || Arc::new(Aligned(1)));
    exact("Arc<[u64]>", || Arc::<[u64]>::from(vec![1u64; 5]));
    exact("Arc<str> empty", || Arc::<str>::from(""));

    // Several handles to one allocation are only counted once.
    exact("(Rc, Rc)", || {
        let shared = Rc::new(String::from("shared"));
        (Rc::clone(&shared), shared)
    });
    exact("Vec<Arc>", || {
        let shared = Arc::new(vec![0u8; 100]);
        vec![Arc::clone(&shared), Arc::clone(&shared), shared]
    });
}

#[test]
fn linked_lists_include_their_links() {
    exact("LinkedList<u8>", || (0..10u8).collect::<LinkedList<_>>());
    exact("LinkedList<u128>", || {
        keys(10).map(u128::from).collect::<LinkedList<_>>()
    });
    exact("LinkedList<()>", || {
        keys(10).map(|_| ()).collect::<LinkedList<_>>()
    });
    exact("LinkedList<Aligned>", || {
        keys(10).map(Aligned).collect::<LinkedList<_>>()
    });
    exact("LinkedList<String>", || {
        keys(3)
            .map(|key| key.to_string())
            .collect::<LinkedList<_>>()
    });
}

#[test]
fn std_hash_tables_match_their_allocation() {
    for capacity in CAPACITIES {
        // Reserved up front.
        exact(&format!("HashSet<u8> with_capacity {capacity}"), || {
            HashSet::<u8>::with_capacity(capacity)
        });
        exact(&format!("HashSet<u64> with_capacity {capacity}"), || {
            HashSet::<u64>::with_capacity(capacity)
        });
        exact(
            &format!("HashMap<u64, [u8; 3]> with_capacity {capacity}"),
            || HashMap::<u64, [u8; 3]>::with_capacity(capacity),
        );
        exact(
            &format!("HashMap<u64, Aligned> with_capacity {capacity}"),
            || HashMap::<u64, Aligned>::with_capacity(capacity),
        );

        // Grown by inserting.
        exact(&format!("HashSet<u64> of {capacity}"), || {
            keys(capacity).collect::<HashSet<_>>()
        });
        exact(&format!("HashMap<u64, u128> of {capacity}"), || {
            keys(capacity)
                .map(|key| (key, u128::from(key)))
                .collect::<HashMap<_, _>>()
        });
        exact(&format!("HashMap<u64, Aligned> of {capacity}"), || {
            keys(capacity)
                .map(|key| (key, Aligned(key)))
                .collect::<HashMap<_, _>>()
        });
        exact(&format!("HashMap<u64, String> of {capacity}"), || {
            keys(capacity)
                .map(|key| (key, key.to_string()))
                .collect::<HashMap<_, _>>()
        });
    }

    // Zero sized entries still need their control bytes.
    exact("HashSet<()>", || {
        std::iter::once(()).collect::<HashSet<_>>()
    });
}

#[test]
fn std_hash_tables_are_a_lower_bound_after_removals() {
    // Removed entries leave tombstones behind, which lower the reported capacity while the
    // allocation stays the same.
    for count in [28, 112, LARGEST] {
        let (allocated, reported) = measure(|| {
            let mut set: HashSet<u64> = keys(count).collect();
            for key in keys(count) {
                set.remove(&key);
            }
            set
        });
        assert!(reported <= allocated, "HashSet emptied from {count}");

        let (allocated, reported) = measure(|| {
            let mut map: HashMap<u64, u64> = keys(count).map(|key| (key, key)).collect();
            map.retain(|key, _| *key % 2 == 0);
            map
        });
        assert!(reported <= allocated, "HashMap retaining half of {count}");
        // The remaining entries are always accounted for.
        assert!(
            reported >= count / 2 * size_of::<(u64, u64)>(),
            "HashMap retaining half of {count}"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "too slow in the interpreter")]
fn trees_are_a_lower_bound() {
    lower_bound("BTreeMap<u64, u64>", || {
        keys(1000).map(|key| (key, key)).collect::<BTreeMap<_, _>>()
    });
    lower_bound("BTreeSet<u64>", || keys(1000).collect::<BTreeSet<_>>());
}

#[test]
fn hashbrown_reports_its_allocation() {
    for capacity in CAPACITIES {
        exact(&format!("hashbrown::HashMap of {capacity}"), || {
            let mut map = hashbrown::HashMap::<u64, Aligned, RandomState>::default();
            map.extend(keys(capacity).map(|key| (key, Aligned(key))));
            map
        });
        exact(&format!("hashbrown::HashSet of {capacity}"), || {
            let mut set = hashbrown::HashSet::<u64, RandomState>::default();
            set.extend(keys(capacity));
            set
        });
        exact(
            &format!("hashbrown::HashTable with_capacity {capacity}"),
            || hashbrown::HashTable::<u64>::with_capacity(capacity),
        );
    }
}

#[test]
fn index_collections_match_their_allocation() {
    for capacity in CAPACITIES {
        exact(
            &format!("IndexMap<u32, u128> with_capacity {capacity}"),
            || {
                indexmap::IndexMap::<u32, u128, RandomState>::with_capacity_and_hasher(
                    capacity,
                    RandomState::new(),
                )
            },
        );
        exact(
            &format!("IndexMap<u64, Aligned> with_capacity {capacity}"),
            || {
                indexmap::IndexMap::<u64, Aligned, RandomState>::with_capacity_and_hasher(
                    capacity,
                    RandomState::new(),
                )
            },
        );
        exact(&format!("IndexSet<u8> with_capacity {capacity}"), || {
            indexmap::IndexSet::<u8, RandomState>::with_capacity_and_hasher(
                capacity,
                RandomState::new(),
            )
        });
        // Grown by inserting.
        exact(&format!("IndexMap<u64, u128> of {capacity}"), || {
            let mut map = indexmap::IndexMap::<u64, u128, RandomState>::default();
            map.extend(keys(capacity).map(|key| (key, u128::from(key))));
            map
        });
        exact(&format!("IndexSet<u64> of {capacity}"), || {
            let mut set = indexmap::IndexSet::<u64, RandomState>::default();
            set.extend(keys(capacity));
            set
        });

        exact(
            &format!("OrderMap<u32, u128> with_capacity {capacity}"),
            || {
                ordermap::OrderMap::<u32, u128, RandomState>::with_capacity_and_hasher(
                    capacity,
                    RandomState::new(),
                )
            },
        );
        exact(&format!("OrderSet<u64> with_capacity {capacity}"), || {
            ordermap::OrderSet::<u64, RandomState>::with_capacity_and_hasher(
                capacity,
                RandomState::new(),
            )
        });
    }
}

#[test]
fn small_buffers_respect_their_inline_storage() {
    exact("SmallVec inline", || {
        smallvec::SmallVec::<[u64; 4]>::from_slice(&[1, 2, 3])
    });
    exact("SmallVec spilled", || {
        keys(100).collect::<smallvec::SmallVec<[u64; 4]>>()
    });
    exact("SmallVec spilled and truncated", || {
        let mut vec: smallvec::SmallVec<[u64; 4]> = keys(100).collect();
        vec.truncate(2);
        vec
    });
    exact("CompactString inline", || {
        compact_str::CompactString::from("short")
    });
    exact("CompactString on the heap", || {
        compact_str::CompactString::from("a string which is too long to be stored inline")
    });
    exact("ThinVec empty", thin_vec::ThinVec::<u64>::new);
    exact("ThinVec<()>", || {
        let mut vec = thin_vec::ThinVec::new();
        vec.push(());
        vec
    });
    exact("ThinVec<Aligned>", || {
        thin_vec::ThinVec::<Aligned>::with_capacity(3)
    });
    exact("ThinVec<String>", || {
        thin_vec::thin_vec![String::from("one"), String::from("two")]
    });
    exact("ThinVec<u64>", || {
        thin_vec::ThinVec::<u64>::with_capacity(10)
    });
}

#[test]
fn byte_buffers_report_their_capacity() {
    exact("BytesMut", || bytes::BytesMut::with_capacity(1000));
    exact("Bytes", || {
        bytes::Bytes::from(vec![0u8; 1000])
    });
    exact("ArcStr", || arcstr::ArcStr::from("a shared string"));
    exact("ArcStr literal", || arcstr::literal!("a static string"));
}

#[test]
#[cfg_attr(miri, ignore = "too slow in the interpreter")]
fn integrations_with_private_bookkeeping_are_a_lower_bound() {
    lower_bound("DashMap<u64, u64>", || {
        let map = dashmap::DashMap::<u64, u64>::new();
        for key in keys(10_000) {
            map.insert(key, key);
        }
        map
    });
    lower_bound("RoaringBitmap of bitset containers", || {
        (0..500_000u32)
            .step_by(3)
            .collect::<roaring::RoaringBitmap>()
    });
    lower_bound("RoaringBitmap of array containers", || {
        (0..500_000u32)
            .step_by(50)
            .collect::<roaring::RoaringBitmap>()
    });
    lower_bound("RoaringBitmap of shrunk array containers", || {
        let mut bitmap: roaring::RoaringBitmap = (0..4000u32).collect();
        bitmap.remove_range(10..4000);
        bitmap
    });
    lower_bound("ConcurrentVec<u64>", || {
        let vec = orx_concurrent_vec::ConcurrentVec::new();
        for key in keys(1000) {
            vec.push(key);
        }
        vec
    });
    lower_bound("Url", || {
        url::Url::parse("https://example.com/a/rather/long/path?with=a&query=string")
            .unwrap_or_else(|err| panic!("the url is valid: {err}"))
    });
}
