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
/// by. This mirrors how `hashbrown` selects its group implementation: 16 bytes with SSE2, 8
/// bytes with NEON, and otherwise a `u64` or `u32` depending on the target.
///
/// `nightly` tells whether the table was built with the `nightly` feature of `hashbrown`, which
/// adds a 16 byte implementation for LSX. The standard library enables it for its own tables,
/// while a `hashbrown` built by `cargo` for another crate does not.
#[cfg(any(feature = "std", feature = "indexmap", feature = "ordermap"))]
const fn group_width(nightly: bool) -> usize {
    if cfg!(miri) {
        GENERIC_GROUP_WIDTH
    } else if cfg!(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
    )) {
        16
    } else if cfg!(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_endian = "little",
    )) {
        8
    } else if nightly && cfg!(all(target_arch = "loongarch64", target_feature = "lsx")) {
        16
    } else {
        GENERIC_GROUP_WIDTH
    }
}

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

/// Estimates the allocation of a hash table of the standard library with room for `capacity`
/// entries.
///
/// The standard library ships its tables precompiled, so on a target whose baseline features
/// differ from the ones this crate is compiled with the result is off by a few bytes per table.
#[cfg(feature = "std")]
pub fn std_hash_table_size(capacity: usize, entry: Layout) -> usize {
    hash_table_size(capacity, entry, group_width(true))
}

/// Estimates the allocation of a `hashbrown` based table with room for `capacity` entries.
///
/// The standard library and `indexmap` do not expose the allocation size of their tables, so it
/// is reconstructed from the layout `hashbrown` is known to use: a power of two number of buckets
/// filled up to 7/8, followed by one control byte per bucket and one trailing group of them.
///
/// The reported capacity is the only available input. It is exact for a table which never had
/// an entry removed, but removals leave tombstones behind which lower the capacity while the
/// allocation stays the same, so the result is a lower bound for such a table, down to zero for
/// one which was emptied.
#[cfg(any(feature = "std", feature = "indexmap", feature = "ordermap"))]
fn hash_table_size(capacity: usize, entry: Layout, group_width: usize) -> usize {
    if capacity == 0 {
        // A table which never held an entry points at a static control group. One which was
        // emptied again reports no capacity either, although its allocation is still alive,
        // which cannot be told apart.
        return 0;
    }

    // Mirrors how `hashbrown` picks the number of buckets for a requested capacity: small
    // tables keep one bucket free, larger ones are filled up to 7/8. A capacity reported by a
    // table yields its own number of buckets again.
    let buckets = if capacity < 4 {
        4
    } else if capacity < 8 {
        8
    } else if capacity < 15 {
        16
    } else {
        // `capacity * 8 / 7`, split up so that it cannot overflow before the division.
        let buckets = (capacity / 7)
            .checked_mul(8)
            .and_then(|buckets| buckets.checked_add(capacity % 7 * 8 / 7))
            .and_then(usize::checked_next_power_of_two);

        // More buckets than the address space can hold.
        let Some(buckets) = buckets else {
            return usize::MAX;
        };
        buckets
    };

    let ctrl_align = entry.align().max(group_width);
    let entries = buckets.saturating_mul(entry.size());
    let ctrl_offset = entries.saturating_add(ctrl_align - 1) & !(ctrl_align - 1);

    ctrl_offset
        .saturating_add(buckets)
        .saturating_add(group_width)
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
        .saturating_add(hash_table_size(
            capacity,
            Layout::new::<usize>(),
            group_width(false),
        ))
}
