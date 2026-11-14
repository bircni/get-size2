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
/// by. `hashbrown` scans 16 bytes with SSE2 and a machine word otherwise.
#[cfg(any(feature = "std", feature = "indexmap", feature = "ordermap"))]
const GROUP_WIDTH: usize = if cfg!(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(miri),
)) {
    16
} else if cfg!(target_arch = "wasm32") {
    8
} else {
    size_of::<usize>()
};

/// Estimates the allocation of a `hashbrown` based table with room for `capacity` entries.
///
/// The standard library and `indexmap` do not expose the allocation size of their tables, so it
/// is reconstructed from the layout `hashbrown` is known to use: a power of two number of buckets
/// filled up to 7/8, followed by one control byte per bucket and one trailing group of them.
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
