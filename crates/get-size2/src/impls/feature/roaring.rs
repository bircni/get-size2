use crate::{GetSize, GetSizeTracker};

// The containers live in a private `Vec<Container>`, so `statistics()` is the only way to learn
// about them. Its byte totals do not describe the allocations though: they report eight times
// the size of a bitset container and twice the capacity of an array container. The heap size is
// therefore derived from the container and value counts instead:
//
// - a bitset container always allocates 1024 words of 64 bits,
// - an array container stores each value as a `u16`,
// - a run container stores each run as two `u16`, which its serialized size covers.
//
// The spare capacity of array and run containers and the `Vec<Container>` itself cannot be
// measured, so the result is a lower bound.

/// The bytes allocated by a bitset container.
const BITSET_CONTAINER_BYTES: u64 = 1024 * 8;

impl GetSize for roaring::RoaringBitmap {
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let s = self.statistics();
        let bytes = u64::from(s.n_bitset_containers)
            .saturating_mul(BITSET_CONTAINER_BYTES)
            .saturating_add(u64::from(s.n_values_array_containers).saturating_mul(2))
            .saturating_add(s.n_bytes_run_containers);

        // At most 65 536 containers of 8 KiB each, which fits everywhere but is saturated to be
        // safe.
        (usize::try_from(bytes).unwrap_or(usize::MAX), tracker)
    }
}

// Mirrors the `BTreeMap<K, V>` impl in `alloc_impls/collections.rs` — per-entry
// stack + heap of both K and V, no node-padding accounting.

impl GetSize for roaring::RoaringTreemap {
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        self.bitmaps()
            .fold((0, tracker), |(size, tracker), (key, bitmap)| {
                let (key_size, tracker) = u32::get_size_with_tracker(&key, tracker);
                let (bm_size, tracker) =
                    roaring::RoaringBitmap::get_size_with_tracker(bitmap, tracker);
                (size + key_size + bm_size, tracker)
            })
    }
}
