use std::mem::size_of;

use get_size2::*;

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
}

#[test]
fn chrono_tz() {
    use chrono::TimeZone;

    let datetime = chrono_tz::UTC
        .with_ymd_and_hms(2014, 7, 8, 9, 10, 11)
        .unwrap();
    assert_eq!(datetime.offset().get_heap_size(), 0);
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

    let mut bytes_mut = bytes::BytesMut::from(BYTES_STR);
    assert_eq!(bytes_mut.get_heap_size(), BYTES_STR.len());
    bytes_mut.truncate(0);
    assert_eq!(bytes_mut.get_heap_size(), 0);
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
}

#[test]
fn test_indexmap() {
    use std::hash::RandomState;

    const VALUE_STR: &str = "A very looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooonng string.";

    let hasher = RandomState::new();

    let mut map = indexmap::IndexMap::with_capacity_and_hasher(1, hasher);
    // The allocation is `capacity * size_of::<(K, V)>()`, which is pointer-width
    // dependent (40 bytes on 64-bit, 20 on 32-bit) — don't hard-code it.
    assert_eq!(
        map.get_heap_size(),
        map.capacity() * size_of::<(&'static str, String)>()
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
    // The allocation is `capacity * size_of::<(K, V)>()`, which is pointer-width
    // dependent (40 bytes on 64-bit, 20 on 32-bit) — don't hard-code it.
    assert_eq!(
        map.get_heap_size(),
        map.capacity() * size_of::<(&'static str, String)>()
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

    // Dense run that fits in one array container: should equal what
    // `statistics()` reports for an array container — `capacity * 4`.
    let dense: roaring::RoaringBitmap = (1..100).collect();
    let stats = dense.statistics();
    assert_eq!(stats.n_containers, 1);
    assert_eq!(stats.n_array_containers, 1);
    assert_eq!(dense.get_heap_size() as u64, stats.n_bytes_array_containers);
    assert!(dense.get_heap_size() > 0);

    // Bitmap container kicks in past ~4096 entries; verify we count
    // the fixed 8 KiB allocation. Equality against the sum of all three
    // flavor totals catches regressions that drop a flavor.
    let wide: roaring::RoaringBitmap = (0..5000).collect();
    let stats = wide.statistics();
    assert!(stats.n_bitset_containers >= 1);
    assert_eq!(
        wide.get_heap_size() as u64,
        stats.n_bytes_array_containers
            + stats.n_bytes_bitset_containers
            + stats.n_bytes_run_containers
    );

    // Run containers are produced by `optimize()` when a container has
    // long dense runs. Verify we count run-container bytes too.
    let mut run = (0..10_000).collect::<roaring::RoaringBitmap>();
    run.optimize();
    let stats = run.statistics();
    assert!(stats.n_run_containers >= 1);
    assert_eq!(
        run.get_heap_size() as u64,
        stats.n_bytes_array_containers
            + stats.n_bytes_bitset_containers
            + stats.n_bytes_run_containers
    );

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

    let b = portable_atomic::AtomicBool::new(true);
    assert_eq!(b.get_heap_size(), 0);
    assert_eq!(b.get_size(), size_of::<portable_atomic::AtomicBool>());
}
