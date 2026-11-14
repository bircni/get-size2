// The implementations are grouped by the crate which provides the implemented types, so that a
// `no_std` build can simply drop the `alloc` and `std` tiers.
mod core_impls;
mod feature;

#[cfg(feature = "alloc")]
mod alloc_impls;
#[cfg(feature = "std")]
mod std_impls;
// Only the heap allocating tiers and integrations have a layout to estimate.
#[cfg(any(feature = "alloc", feature = "indexmap", feature = "ordermap"))]
mod layout;
