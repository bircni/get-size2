use crate::{GetSize, GetSizeTracker};

// Container allocations live in a private `Vec<Container>`; `statistics()`
// is the only public proxy for their byte totals. Per-container key/tag
// fields and the outer `Vec<Container>` slots are not counted — bounded
// by 65 536 containers per bitmap, negligible vs. up to ~512 MiB of
// container payload.

impl GetSize for roaring::RoaringBitmap {
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let s = self.statistics();
        let bytes =
            s.n_bytes_array_containers + s.n_bytes_bitset_containers + s.n_bytes_run_containers;
        // u64 → usize: bounded ~512 MiB so fits everywhere; saturate
        // defensively in release, surface the invariant breach in debug.
        debug_assert!(bytes <= usize::MAX as u64);
        (usize::try_from(bytes).unwrap_or(usize::MAX), tracker)
    }
}

// Mirrors the `BTreeMap<K, V>` impl in `collections.rs` — per-entry
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
