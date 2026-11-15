// The crate itself is `no_std`, so the tests have to import the `alloc` based prelude items
// explicitly.
use std::string::String;
use std::vec;

use std::mem::size_of;

use get_size2::*;

const ARCSTR_HEADER: usize = 2 * size_of::<usize>();

#[test]
fn arcstr() {
    let value = arcstr::ArcStr::from("Hello, world");
    assert_eq!(value.get_heap_size(), ARCSTR_HEADER + "Hello, world".len());
    assert_eq!(
        value.get_size(),
        size_of::<arcstr::ArcStr>() + ARCSTR_HEADER + value.len()
    );
}

#[test]
fn arcstr_shared() {
    const TEXT: &str = "Hello world";
    let value = arcstr::ArcStr::from(TEXT);
    let shared = (arcstr::ArcStr::clone(&value), arcstr::ArcStr::clone(&value));
    let (size, _) = shared.get_heap_size_with_tracker(NoTracker::new(true));
    assert_eq!(size, (ARCSTR_HEADER + TEXT.len()) * 2);
    assert_eq!(shared.get_heap_size(), ARCSTR_HEADER + TEXT.len());
    let (size, tracker) = shared.get_heap_size_with_tracker(StandardTracker::new());
    assert_eq!(size, ARCSTR_HEADER + TEXT.len());

    let (size, tracker) = value.get_heap_size_with_tracker(tracker);
    assert_eq!(size, 0);
    let separate = arcstr::ArcStr::from(TEXT);
    let (size, _) = separate.get_heap_size_with_tracker(tracker);
    assert_eq!(size, ARCSTR_HEADER + TEXT.len());

    let (size, _) = value.get_heap_size_with_tracker(NoTracker::new(false));
    assert_eq!(size, 0);
}

#[test]
fn arcstr_static() {
    for value in [
        arcstr::ArcStr::new(),
        arcstr::ArcStr::default(),
        arcstr::ArcStr::from(""),
        arcstr::literal!("Hello world"),
    ] {
        assert_eq!(value.get_heap_size(), 0);
        assert_eq!(value.get_size(), size_of::<arcstr::ArcStr>());
        assert_eq!(arcstr::ArcStr::clone(&value).get_heap_size(), 0);
    }
}

#[test]
fn arcstr_substr() {
    let parent = arcstr::ArcStr::from("Hello world");
    let first = parent.substr(..5);
    let second = parent.substr(6..);
    assert_eq!(first.get_heap_size(), ARCSTR_HEADER + parent.len());
    assert_eq!(
        first.get_size(),
        size_of::<arcstr::Substr>() + ARCSTR_HEADER + parent.len()
    );
    assert_eq!(second.get_heap_size(), ARCSTR_HEADER + parent.len());

    let (size, tracker) = (first, second).get_heap_size_with_tracker(StandardTracker::new());
    assert_eq!(size, ARCSTR_HEADER + parent.len());
    let (size, _) = parent.get_heap_size_with_tracker(tracker);
    assert_eq!(size, 0);
}

#[test]
fn arcstr_substr_static() {
    assert_eq!(arcstr::Substr::new().get_heap_size(), 0);
    assert_eq!(
        arcstr::literal!("Hello world").substr(6..).get_heap_size(),
        0
    );
}

#[test]
fn chrono() {
    use chrono::TimeZone;

    let timedelta = chrono::TimeDelta::seconds(5);
    assert_eq!(timedelta.get_heap_size(), 0);

    let datetime = chrono::Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap();
    assert_eq!(datetime.naive_utc().get_heap_size(), 0);
    assert_eq!(datetime.naive_utc().date().get_heap_size(), 0);
    assert_eq!(datetime.naive_utc().time().get_heap_size(), 0);
    assert_eq!(datetime.timezone().get_heap_size(), 0);
    assert_eq!(datetime.fixed_offset().timezone().get_heap_size(), 0);
    assert_eq!(datetime.get_heap_size(), 0);

    assert_eq!(chrono::Month::July.get_heap_size(), 0);
    assert_eq!(chrono::Weekday::Tue.get_heap_size(), 0);
    assert_eq!(
        chrono::Weekday::Tue.get_size(),
        size_of::<chrono::Weekday>()
    );
}

#[test]
fn chrono_tz() {
    use chrono::TimeZone;

    let datetime = chrono_tz::UTC
        .with_ymd_and_hms(2014, 7, 8, 9, 10, 11)
        .unwrap();
    assert_eq!(datetime.offset().get_heap_size(), 0);
    assert_eq!(datetime.timezone().get_heap_size(), 0);
    assert_eq!(datetime.get_heap_size(), 0);
    assert_eq!(
        chrono_tz::Europe::Berlin.get_size(),
        size_of::<chrono_tz::Tz>()
    );
}

#[test]
fn url() {
    const URL_STR: &str = "https://example.com/path?a=b&c=d";

    let url = url::Url::parse(URL_STR).unwrap();
    assert_eq!(url.get_heap_size(), URL_STR.len());
}

#[test]
fn bytes() {
    const BYTES_STR: &str = "Hello world";

    let bytes = bytes::Bytes::from(BYTES_STR);
    assert_eq!(bytes.get_heap_size(), BYTES_STR.len());

    assert_eq!(bytes::Bytes::new().get_heap_size(), 0);

    // Regression test: views were deduplicated by their start address, which dropped the larger
    // of two views starting at the same position.
    let buffer = bytes::Bytes::from(vec![0u8; 1000]);
    let views = (buffer.slice(0..1), buffer);
    assert_eq!(views.get_heap_size(), 1001);

    let mut bytes_mut = bytes::BytesMut::from(BYTES_STR);
    assert_eq!(bytes_mut.get_heap_size(), BYTES_STR.len());

    // Regression test: the length was reported instead of the allocated capacity.
    bytes_mut.truncate(0);
    assert_eq!(bytes_mut.get_heap_size(), bytes_mut.capacity());
    let bytes_mut = bytes::BytesMut::with_capacity(1000);
    assert!(bytes_mut.get_heap_size() >= 1000);
}

#[test]
fn compact_str() {
    const STR: &str = "Hello world";
    const LONG_STR: &str = "A much looooonger string that exceeds 24 bytes.";

    let value = compact_str::CompactString::from(STR);
    assert_eq!(value.get_heap_size(), 0);

    let mut value = compact_str::CompactString::from(LONG_STR);
    assert_eq!(value.get_heap_size(), value.capacity());

    value.shrink_to_fit();

    assert_eq!(value.len(), value.capacity());
    assert_eq!(value.get_heap_size(), LONG_STR.len());
}

#[test]
fn test_dashmap() {
    const VALUE_STR: &str = "Hello world";

    let map: dashmap::DashMap<i32, String> = dashmap::DashMap::new();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let set: dashmap::DashSet<String> = dashmap::DashSet::new();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn test_half() {
    let a = half::f16::from_f32(1.5);
    assert_eq!(a.get_heap_size(), 0);
    assert_eq!(a.get_size(), size_of::<half::f16>());

    let b = half::bf16::from_f32(1.5);
    assert_eq!(b.get_heap_size(), 0);
    assert_eq!(b.get_size(), size_of::<half::bf16>());
}

#[test]
fn hashbrown() {
    use std::hash::{BuildHasher, RandomState};

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = hashbrown::HashTable::new();
    assert_eq!(map.get_heap_size(), 0);
    map.insert_unique(
        hasher.hash_one(VALUE_STR),
        String::from(VALUE_STR),
        |value| hasher.hash_one(value),
    );
    assert!(map.get_heap_size() >= size_of::<String>() + VALUE_STR.len());

    let mut map = hashbrown::HashMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = hashbrown::HashSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn smallvec() {
    const ITEM_STR: &str = "Hello world";
    let mut vec = smallvec::SmallVec::<[String; 2]>::from([String::new(), String::from(ITEM_STR)]);

    assert_eq!(vec.get_heap_size(), ITEM_STR.len());
    vec.push(String::new());

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len() + std::mem::size_of::<String>() * vec.capacity()
    );

    vec.shrink_to_fit();

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len() + std::mem::size_of::<String>() * 3
    );
}

#[test]
fn thin_vec() {
    const ITEM_STR: &str = "Hello world";

    assert_eq!(thin_vec::ThinVec::<String>::default().get_heap_size(), 0);

    let mut vec = thin_vec::ThinVec::<String>::from([String::new(), String::from(ITEM_STR)]);
    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len()
            + std::mem::size_of::<String>() * vec.capacity()
            + std::mem::size_of::<usize>() * 2
    );

    vec.shrink_to_fit();

    assert_eq!(
        vec.get_heap_size(),
        ITEM_STR.len()
            + std::mem::size_of::<String>() * vec.len()
            + std::mem::size_of::<usize>() * 2
    );

    // Regression test: zero sized elements never allocate, and the header is padded up to the
    // alignment of the elements.
    let mut zero_sized = thin_vec::ThinVec::<()>::new();
    zero_sized.push(());
    assert_eq!(zero_sized.get_heap_size(), 0);

    let aligned = thin_vec::ThinVec::<u128>::with_capacity(4);
    assert_eq!(
        aligned.get_heap_size(),
        (2 * size_of::<usize>()).max(align_of::<u128>()) + aligned.capacity() * size_of::<u128>()
    );
}

#[test]
fn test_indexmap() {
    use std::hash::RandomState;

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = indexmap::IndexMap::with_capacity_and_hasher(1, hasher);
    // The entries are stored next to their hash, and indexed by a hash table of positions. All
    // of it is pointer-width dependent, so don't hard-code it.
    assert_eq!(
        map.get_heap_size(),
        map.capacity() * size_of::<(usize, &'static str, String)>()
            + hashbrown::HashTable::<usize>::with_capacity(map.capacity()).allocation_size()
    );
    map.insert(VALUE_STR, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(&'static str, String)>() + VALUE_STR.len());

    let mut map = indexmap::IndexMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = indexmap::IndexSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn test_parking_lot() {
    use std::sync::Arc;

    const S: &str = "Hello world";

    let v = vec![String::from(S)];
    let expected = size_of::<String>() + S.len();

    let m = parking_lot::Mutex::new(v.clone());
    assert_eq!(m.get_heap_size(), expected);

    let r = parking_lot::RwLock::new(v);
    assert_eq!(r.get_heap_size(), expected);

    let tracker = Arc::new(parking_lot::Mutex::new(StandardTracker::new()));
    let (size, _) = r.get_heap_size_with_tracker(tracker);
    assert_eq!(size, expected);
}

#[test]
fn test_ordermap() {
    use std::hash::RandomState;

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = ordermap::OrderMap::with_capacity_and_hasher(1, hasher);
    // The entries are stored next to their hash, and indexed by a hash table of positions. All
    // of it is pointer-width dependent, so don't hard-code it.
    assert_eq!(
        map.get_heap_size(),
        map.capacity() * size_of::<(usize, &'static str, String)>()
            + hashbrown::HashTable::<usize>::with_capacity(map.capacity()).allocation_size()
    );
    map.insert(VALUE_STR, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(&'static str, String)>() + VALUE_STR.len());

    let mut map = ordermap::OrderMap::<i32, String, RandomState>::default();
    assert_eq!(map.get_heap_size(), 0);
    map.insert(0, String::from(VALUE_STR));
    assert!(map.get_heap_size() >= size_of::<(i32, String)>() + VALUE_STR.len());

    let mut set = ordermap::OrderSet::<String, RandomState>::default();
    assert_eq!(set.get_heap_size(), 0);
    set.insert(String::from(VALUE_STR));
    assert!(set.get_heap_size() >= size_of::<String>() + VALUE_STR.len());
}

#[test]
fn test_roaring_bitmap() {
    // Empty bitmap: no containers, no allocations.
    let empty = roaring::RoaringBitmap::new();
    assert_eq!(empty.get_heap_size(), 0);

    // An array container stores each of its values as a `u16`.
    let dense: roaring::RoaringBitmap = (1..100).collect();
    let stats = dense.statistics();
    assert_eq!(stats.n_containers, 1);
    assert_eq!(stats.n_array_containers, 1);
    assert!(dense.get_heap_size() >= 99 * size_of::<u16>());
    // Regression test: the statistics count the capacity as if the values were `u32`.
    assert_eq!(
        dense.get_heap_size() as u64,
        stats.n_bytes_array_containers / 2
    );

    // A bitset container kicks in past 4096 entries and always allocates 8 KiB. Regression
    // test: the byte total of `statistics()` reports eight times as much.
    let wide: roaring::RoaringBitmap = (0..5000).collect();
    let stats = wide.statistics();
    assert_eq!(stats.n_bitset_containers, 1);
    assert_eq!(wide.get_heap_size(), 8192);

    // Run containers are produced by `optimize()` when a container has long dense runs, and
    // store each run as two `u16`.
    let mut run = (0..10_000).collect::<roaring::RoaringBitmap>();
    run.optimize();
    let stats = run.statistics();
    assert_eq!(stats.n_run_containers, 1);
    assert!(run.get_heap_size() >= 2 * size_of::<u16>());
    assert!(run.get_heap_size() < 16);

    // All three flavors add up.
    let mut mixed: roaring::RoaringBitmap = (0..5000).collect();
    mixed.extend((1 << 16)..(1 << 16) + 10);
    assert!(mixed.get_heap_size() >= 8192 + 10 * size_of::<u16>());
    assert!(mixed.get_heap_size() < 8192 + 100);

    // Tracker-variant: threading a real tracker through must return the
    // same size as the no-tracker path.
    let tracker = StandardTracker::new();
    let (size, _) = wide.get_heap_size_with_tracker(tracker);
    assert_eq!(size, wide.get_heap_size());
}

#[test]
fn test_roaring_treemap() {
    let empty = roaring::RoaringTreemap::new();
    assert_eq!(empty.get_heap_size(), 0);

    // Two high-32-bit partitions → two inner RoaringBitmaps. The second
    // partition holds many values so the per-bitmap heap size is
    // non-trivial (exercises propagation through the BTreeMap walk).
    let mut tm = roaring::RoaringTreemap::new();
    tm.insert(1);
    for v in 0..5000u64 {
        tm.insert((1u64 << 32) + v);
    }
    let mut bitmap_count = 0;
    let mut expected_inner: usize = 0;
    for (_, bitmap) in tm.bitmaps() {
        bitmap_count += 1;
        expected_inner +=
            bitmap.get_heap_size() + size_of::<u32>() + size_of::<roaring::RoaringBitmap>();
    }
    assert_eq!(bitmap_count, 2);
    assert_eq!(tm.get_heap_size(), expected_inner);

    // Tracker-variant: threading a tracker through the BTreeMap walk
    // must match the no-tracker path.
    let tracker = StandardTracker::new();
    let (size, _) = tm.get_heap_size_with_tracker(tracker);
    assert_eq!(size, tm.get_heap_size());
}

#[test]
fn test_orx_concurrent_vec() {
    use orx_concurrent_vec::{ConcurrentElement, ConcurrentVec};

    const S: &str = "Hello world";

    let empty: ConcurrentVec<u32> = ConcurrentVec::new();
    assert_eq!(
        empty.get_heap_size(),
        empty.capacity() * size_of::<ConcurrentElement<u32>>()
    );

    let vec: ConcurrentVec<u32> = ConcurrentVec::new();
    vec.extend(0u32..16);
    assert_eq!(
        vec.get_heap_size(),
        vec.capacity() * size_of::<ConcurrentElement<u32>>()
    );

    let vec: ConcurrentVec<String> = ConcurrentVec::new();
    vec.push(String::from(S));
    vec.push(String::from(S));
    let expected = vec.capacity() * size_of::<ConcurrentElement<String>>() + S.len() * 2;
    assert_eq!(vec.get_heap_size(), expected);

    let tracker = StandardTracker::new();
    let (size, _) = vec.get_heap_size_with_tracker(tracker);
    assert_eq!(size, vec.get_heap_size());
}

#[test]
fn test_portable_atomic() {
    // Atomics are stack-only, so heap size is 0 and total size equals the
    // type's own size. `AtomicU64`/`AtomicI64`/`AtomicU128`/`AtomicI128` are
    // the whole point of the feature: they exist here even on targets without
    // native 64/128-bit atomics.
    let u64 = portable_atomic::AtomicU64::new(1);
    assert_eq!(u64.get_heap_size(), 0);
    assert_eq!(u64.get_size(), size_of::<portable_atomic::AtomicU64>());

    let i64 = portable_atomic::AtomicI64::new(-1);
    assert_eq!(i64.get_heap_size(), 0);
    assert_eq!(i64.get_size(), size_of::<portable_atomic::AtomicI64>());

    let u128 = portable_atomic::AtomicU128::new(1);
    assert_eq!(u128.get_heap_size(), 0);
    assert_eq!(u128.get_size(), size_of::<portable_atomic::AtomicU128>());

    let ptr = portable_atomic::AtomicPtr::new(std::ptr::null_mut::<u8>());
    assert_eq!(ptr.get_heap_size(), 0);
    assert_eq!(ptr.get_size(), size_of::<portable_atomic::AtomicPtr<u8>>());

    let b = portable_atomic::AtomicBool::new(true);
    assert_eq!(b.get_heap_size(), 0);
    assert_eq!(b.get_size(), size_of::<portable_atomic::AtomicBool>());
}

#[test]
fn uuid() {
    for value in [uuid::Uuid::nil(), uuid::Uuid::from_u128(0x1234)] {
        assert_eq!(value.get_heap_size(), 0);
        assert_eq!(value.get_size(), size_of::<uuid::Uuid>());
        assert_eq!(value.get_size(), 16);

        let (size, tracker) = value.get_heap_size_with_tracker(NoTracker::new(false));
        assert_eq!(size, 0);
        assert!(!tracker.answer());
    }
}

#[test]
fn smallvec_spilled_stays_on_heap() {
    // Regression test: a spilled vector which shrank below its inline size reported no heap.
    let mut vec: smallvec::SmallVec<[u64; 4]> = (0..100).collect();
    vec.truncate(2);

    assert!(vec.spilled());
    assert_eq!(vec.get_heap_size(), vec.capacity() * size_of::<u64>());
}

#[test]
fn parking_lot_trackers_deduplicate() {
    use std::rc::Rc;
    use std::sync::Arc;

    // Every lock flavor has to forward to the tracker it guards.
    fn assert_tracks<T: GetSizeTracker>(tracker: T) {
        let shared = Rc::new(String::from("shared"));
        let pair = (Rc::clone(&shared), Rc::clone(&shared));

        let (size, tracker) = pair.get_heap_size_with_tracker(tracker);
        assert_eq!(size, shared.get_heap_size());

        let (size, _) = shared.get_heap_size_with_tracker(tracker);
        assert_eq!(size, 0);
    }

    assert_tracks(parking_lot::Mutex::new(StandardTracker::new()));
    assert_tracks(parking_lot::RwLock::new(StandardTracker::new()));
    assert_tracks(Arc::new(parking_lot::Mutex::new(StandardTracker::new())));
    assert_tracks(Arc::new(parking_lot::RwLock::new(StandardTracker::new())));
}

#[test]
fn parking_lot_fair_and_reentrant_mutex() {
    let fair = parking_lot::FairMutex::new(String::from("locked"));
    assert_eq!(fair.get_heap_size(), "locked".len());

    let reentrant = parking_lot::ReentrantMutex::new(String::from("locked"));
    assert_eq!(reentrant.get_heap_size(), "locked".len());

    // The holding thread may lock again, so measuring does not deadlock.
    let guard = reentrant.lock();
    assert_eq!(reentrant.get_heap_size(), "locked".len());
    drop(guard);
}

#[test]
fn index_collections_need_no_hasher_bounds() {
    // Regression test: the map required `BuildHasher` while the set did not.
    struct NotAHasher;

    let map = indexmap::IndexMap::<u64, String, NotAHasher>::with_hasher(NotAHasher);
    assert_eq!(map.get_heap_size(), 0);
    let set = indexmap::IndexSet::<u64, NotAHasher>::with_hasher(NotAHasher);
    assert_eq!(set.get_heap_size(), 0);

    let map = ordermap::OrderMap::<u64, String, NotAHasher>::with_hasher(NotAHasher);
    assert_eq!(map.get_heap_size(), 0);
    let set = ordermap::OrderSet::<u64, NotAHasher>::with_hasher(NotAHasher);
    assert_eq!(set.get_heap_size(), 0);
}

#[test]
fn std_hash_tables_stay_a_lower_bound_after_removals() {
    // Removed entries leave tombstones behind, which lower the reported capacity while the
    // allocation stays the same. The estimate must then err on the low side.
    let mut set: std::collections::HashSet<u64> = (0..112).collect();
    let allocated = set.get_heap_size();
    for value in 0..112 {
        set.remove(&value);
    }
    assert!(set.get_heap_size() <= allocated);

    let mut map: std::collections::HashMap<u64, u64> = (0..28).map(|key| (key, key)).collect();
    let allocated = map.get_heap_size();
    map.retain(|_, _| false);
    assert!(map.get_heap_size() <= allocated);
}

#[test]
fn index_collections_store_flat_buckets() {
    use std::hash::RandomState;

    // Regression test: the entry was measured as `(usize, (K, V))`, which pads an over-aligned
    // value twice.
    let entry = size_of::<(usize, u32, u128)>();
    let table = |capacity| hashbrown::HashTable::<usize>::with_capacity(capacity).allocation_size();

    let map = indexmap::IndexMap::<u32, u128, RandomState>::with_capacity_and_hasher(
        1000,
        RandomState::new(),
    );
    assert_eq!(
        map.get_heap_size(),
        map.capacity() * entry + table(map.capacity())
    );

    let map = ordermap::OrderMap::<u32, u128, RandomState>::with_capacity_and_hasher(
        1000,
        RandomState::new(),
    );
    assert_eq!(
        map.get_heap_size(),
        map.capacity() * entry + table(map.capacity())
    );

    let set =
        indexmap::IndexSet::<u128, RandomState>::with_capacity_and_hasher(1000, RandomState::new());
    assert_eq!(
        set.get_heap_size(),
        set.capacity() * size_of::<(usize, u128)>() + table(set.capacity())
    );
}
