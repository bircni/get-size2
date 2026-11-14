//! Size estimates for allocations whose exact layout is private to the owning type.

use core::alloc::Layout;

/// The size of an allocation storing a header of type `H` in front of `value`.
///
/// Used for the reference counters of `Rc`/`Arc` and the links of a `LinkedList` node. The
/// padding between the header and the value, and at the end of the allocation, is included.
#[cfg(feature = "alloc")]
pub fn size_with_header<H, T: ?Sized>(value: &T) -> usize {
    let header = Layout::new::<H>();
    let value = Layout::for_value(value);

    header.extend(value).map_or_else(
        // Unreachable for an existing value, but a too low result beats a panic.
        |_| header.size() + value.size(),
        |(layout, _offset)| layout.pad_to_align().size(),
    )
}

/// The number of control bytes a `hashbrown` table scans at once, which it also over-allocates
/// by. This mirrors how `hashbrown` selects its group implementation: 16 bytes with SSE2 or LSX,
/// 8 bytes with NEON, and otherwise a `u64` or `u32` depending on the target.
///
/// The standard library ships its table precompiled, so on a target whose baseline features
/// differ from the ones this crate is compiled with the value is off by a few bytes per table.
#[cfg(any(feature = "std", feature = "indexmap", feature = "ordermap"))]
const GROUP_WIDTH: usize = if cfg!(miri) {
    GENERIC_GROUP_WIDTH
} else if cfg!(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
    ),
    all(target_arch = "loongarch64", target_feature = "lsx"),
)) {
    16
} else if cfg!(all(
    target_arch = "aarch64",
    target_feature = "neon",
    target_endian = "little",
)) {
    8
} else {
    GENERIC_GROUP_WIDTH
};

/// The group width of the portable `hashbrown` implementation.
#[cfg(any(feature = "std", feature = "indexmap", feature = "ordermap"))]
const GENERIC_GROUP_WIDTH: usize = if cfg!(any(
    target_pointer_width = "64",
    target_arch = "aarch64",
    target_arch = "x86_64",
    target_arch = "wasm32",
)) {
    8
} else {
    4
};

/// Estimates the allocation of a `hashbrown` based table with room for `capacity` entries.
///
/// The standard library and `indexmap` do not expose the allocation size of their tables, so it
/// is reconstructed from the layout `hashbrown` is known to use: a power of two number of buckets
/// filled up to 7/8, followed by one control byte per bucket and one trailing group of them.
///
/// The reported capacity is the only available input. It is exact for a table which never had
/// an entry removed, but removals leave tombstones behind which lower the capacity while the
/// allocation stays the same, so the result is a lower bound for such a table.
#[cfg(any(feature = "std", feature = "indexmap", feature = "ordermap"))]
pub fn hash_table_size(capacity: usize, entry: Layout) -> usize {
    if capacity == 0 {
        // An empty table points at a static control group.
        return 0;
    }

    // A zero sized entry type reports an unbounded capacity, and occupies no buckets anyway.
    let buckets = capacity
        .checked_mul(8)
        .map(|scaled| scaled / 7)
        .and_then(usize::checked_next_power_of_two)
        .map_or(4, |buckets| buckets.max(4));

    let ctrl_align = entry.align().max(GROUP_WIDTH);
    let entries = buckets.saturating_mul(entry.size());
    let ctrl_offset = entries.saturating_add(ctrl_align - 1) & !(ctrl_align - 1);

    ctrl_offset
        .saturating_add(buckets)
        .saturating_add(GROUP_WIDTH)
}

/// An entry of an `indexmap` or `ordermap` collection, which stores the hash next to the key
/// and the value. Mirrored here since the size of the real type is private.
#[cfg(any(feature = "indexmap", feature = "ordermap"))]
#[expect(dead_code, reason = "only the layout of this type is of interest")]
struct IndexBucket<K, V> {
    hash: usize,
    key: K,
    value: V,
}

/// Estimates the allocations of an `indexmap` or `ordermap` collection with room for `capacity`
/// entries. Sets store `()` as their value.
///
/// The entries live in a `Vec`, and a separate hash table maps the hashes to their position
/// inside that `Vec`.
#[cfg(any(feature = "indexmap", feature = "ordermap"))]
pub fn index_map_size<K, V>(capacity: usize) -> usize {
    capacity
        .saturating_mul(size_of::<IndexBucket<K, V>>())
        .saturating_add(hash_table_size(capacity, Layout::new::<usize>()))
}
