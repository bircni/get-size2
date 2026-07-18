use crate::GetSize;

// `portable-atomic` provides atomic types that work on every target, including
// 32-bit ones (e.g. ppc32) that lack native 64-bit atomics and therefore have
// no `std::sync::atomic::AtomicU64`/`AtomicI64` (see issue #54). These impls let
// crates that reach for `portable_atomic` on such targets still measure size.
//
// Like their std counterparts in `impls/primitives.rs`, these are stack-only
// types, so the default `get_heap_size` (returning 0) is correct.

impl GetSize for portable_atomic::AtomicBool {}
impl GetSize for portable_atomic::AtomicI8 {}
impl GetSize for portable_atomic::AtomicI16 {}
impl GetSize for portable_atomic::AtomicI32 {}
impl GetSize for portable_atomic::AtomicI64 {}
impl GetSize for portable_atomic::AtomicI128 {}
impl GetSize for portable_atomic::AtomicIsize {}
impl GetSize for portable_atomic::AtomicU8 {}
impl GetSize for portable_atomic::AtomicU16 {}
impl GetSize for portable_atomic::AtomicU32 {}
impl GetSize for portable_atomic::AtomicU64 {}
impl GetSize for portable_atomic::AtomicU128 {}
impl GetSize for portable_atomic::AtomicUsize {}
