use crate::{GetSize, GetSizeTracker};

// `RoaringBitmap` stores 32-bit integers in "containers" of three
// flavors (array, bitset, run), held in a private `Vec<Container>`.
// Container allocations aren't reachable from outside the crate, but
// `RoaringBitmap::statistics()` walks them internally and reports
// per-flavor byte totals (capacity-based for array/run containers,
// fixed 8 KiB for bitset). We sum those.
//
// Not counted (both small relative to container contents):
// - Outer `Vec<Container>` slot allocation. `Container` is `pub(crate)`,
//   so its size isn't accessible here. Max 65 536 containers per bitmap.
// - Per-container key/tag fields.

impl GetSize for roaring::RoaringBitmap {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        let s = self.statistics();
        let bytes =
            s.n_bytes_array_containers + s.n_bytes_bitset_containers + s.n_bytes_run_containers;
        // A single RoaringBitmap is bounded by 65 536 containers × 8 KiB
        // (~512 MiB), so this fits in `usize` on every supported target.
        // Saturate as a defensive fallback rather than panicking.
        (usize::try_from(bytes).unwrap_or(usize::MAX), tracker)
    }
}

// `RoaringTreemap` is internally `BTreeMap<u32, RoaringBitmap>`. The
// map is private, but `bitmaps()` is a public iterator over
// `(u32, &RoaringBitmap)` pairs, which is enough to sum the contained
// bitmaps. Mirrors the `BTreeMap<K, V>` impl in `collections.rs`:
// per-entry it adds the stack-size + heap-size of both K and V, and
// does not attempt to account for BTreeMap node padding.

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
