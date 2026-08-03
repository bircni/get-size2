use core::convert::Infallible;
use core::marker::{PhantomData, PhantomPinned};
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};
use core::sync::atomic::Ordering;
// An atomic type only exists on targets whose atomic support matches its width. Bare metal
// targets in particular may lack some of them, as do 32-bit targets (e.g. ppc32) for the 64-bit
// atomics. See issue #54.
#[cfg(target_has_atomic = "8")]
use core::sync::atomic::{AtomicBool, AtomicI8, AtomicU8};
#[cfg(target_has_atomic = "16")]
use core::sync::atomic::{AtomicI16, AtomicU16};
#[cfg(target_has_atomic = "32")]
use core::sync::atomic::{AtomicI32, AtomicU32};
#[cfg(target_has_atomic = "64")]
use core::sync::atomic::{AtomicI64, AtomicU64};
#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic::{AtomicIsize, AtomicUsize};
use core::time::Duration;

use crate::GetSize;

impl GetSize for () {}
impl GetSize for bool {}
impl GetSize for u8 {}
impl GetSize for u16 {}
impl GetSize for u32 {}
impl GetSize for u64 {}
impl GetSize for u128 {}
impl GetSize for usize {}
impl GetSize for NonZeroU8 {}
impl GetSize for NonZeroU16 {}
impl GetSize for NonZeroU32 {}
impl GetSize for NonZeroU64 {}
impl GetSize for NonZeroU128 {}
impl GetSize for NonZeroUsize {}
impl GetSize for i8 {}
impl GetSize for i16 {}
impl GetSize for i32 {}
impl GetSize for i64 {}
impl GetSize for i128 {}
impl GetSize for isize {}
impl GetSize for NonZeroI8 {}
impl GetSize for NonZeroI16 {}
impl GetSize for NonZeroI32 {}
impl GetSize for NonZeroI64 {}
impl GetSize for NonZeroI128 {}
impl GetSize for NonZeroIsize {}
impl GetSize for f32 {}
impl GetSize for f64 {}
impl GetSize for char {}

#[cfg(target_has_atomic = "8")]
impl GetSize for AtomicBool {}
#[cfg(target_has_atomic = "8")]
impl GetSize for AtomicI8 {}
#[cfg(target_has_atomic = "16")]
impl GetSize for AtomicI16 {}
#[cfg(target_has_atomic = "32")]
impl GetSize for AtomicI32 {}
#[cfg(target_has_atomic = "64")]
impl GetSize for AtomicI64 {}
#[cfg(target_has_atomic = "ptr")]
impl GetSize for AtomicIsize {}
#[cfg(target_has_atomic = "8")]
impl GetSize for AtomicU8 {}
#[cfg(target_has_atomic = "16")]
impl GetSize for AtomicU16 {}
#[cfg(target_has_atomic = "32")]
impl GetSize for AtomicU32 {}
#[cfg(target_has_atomic = "64")]
impl GetSize for AtomicU64 {}
#[cfg(target_has_atomic = "ptr")]
impl GetSize for AtomicUsize {}
impl GetSize for Ordering {}
impl GetSize for core::cmp::Ordering {}

impl GetSize for Infallible {}
impl<T> GetSize for PhantomData<T> {}
impl GetSize for PhantomPinned {}

impl GetSize for Duration {}
