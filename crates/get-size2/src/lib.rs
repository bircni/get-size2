#![doc = include_str!("./lib.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::convert::Infallible;
use std::marker::{PhantomData, PhantomPinned};
use std::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::ptr::NonNull;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::atomic::{
    AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16,
    AtomicU32, AtomicU64, AtomicUsize, Ordering,
};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak as ArcWeak};
use std::time::{Duration, Instant, SystemTime};

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use get_size_derive2::*;

mod tracker;
pub use tracker::*;
#[cfg(test)]
mod test;

/// Determines how many bytes the object occupies inside the heap.
pub fn heap_size<T: GetSize>(value: &T) -> usize {
    value.get_heap_size()
}

/// Determine the size in bytes an object occupies inside RAM.
pub trait GetSize: Sized {
    /// Determines how may bytes this object occupies inside the stack.
    ///
    /// The default implementation uses [`std::mem::size_of`] and should work for almost all types.
    #[must_use]
    fn get_stack_size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Determines how many bytes this object occupies inside the heap.
    ///
    /// The default implementation simply delegates to [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker)
    /// with a noop tracker. This method is not meant to be implemented directly, and only exists for convenience.
    fn get_heap_size(&self) -> usize {
        let mut tracker = NoTracker::new(true);
        self.get_heap_size_with_tracker(&mut tracker)
    }

    /// Determines how many bytes this object occupies inside the heap while using a `tracker`.
    ///
    /// The default implementation returns 0, assuming the object is fully allocated on the stack.
    /// It must be adjusted as appropriate for objects which hold data inside the heap.
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        0
    }

    /// Determines the total size of the object.
    ///
    /// The default implementation simply adds up the results of [`get_stack_size`](Self::get_stack_size)
    /// and [`get_heap_size`](Self::get_heap_size) and is not meant to be changed.
    fn get_size(&self) -> usize {
        Self::get_stack_size() + GetSize::get_heap_size(self)
    }

    /// Determines the total size of the object while using a `tracker`.
    ///
    /// The default implementation simply adds up the results of [`get_stack_size`](Self::get_stack_size)
    /// and [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker) and is not meant to
    /// be changed.
    fn get_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        Self::get_stack_size() + self.get_heap_size_with_tracker(tracker)
    }
}

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

impl GetSize for AtomicBool {}
impl GetSize for AtomicI8 {}
impl GetSize for AtomicI16 {}
impl GetSize for AtomicI32 {}
impl GetSize for AtomicI64 {}
impl GetSize for AtomicIsize {}
impl GetSize for AtomicU8 {}
impl GetSize for AtomicU16 {}
impl GetSize for AtomicU32 {}
impl GetSize for AtomicU64 {}
impl GetSize for AtomicUsize {}
impl GetSize for Ordering {}

impl GetSize for std::cmp::Ordering {}

impl GetSize for Infallible {}
impl<T> GetSize for PhantomData<T> {}
impl GetSize for PhantomPinned {}

impl GetSize for Instant {}
impl GetSize for Duration {}
impl GetSize for SystemTime {}

/// This macro is similar to the derive macro; Generate a `GetSize` impl that
/// adds the heap sizes of the fields that are specified. However, since we want
/// to implement this also for third-party types where we cannot add the derive
/// macro, we go this way.
///
/// # Examples
/// ```ignore
/// impl_sum_of_fields!(Range, start, end);
/// impl_sum_of_fields!(Person, name, address, phone);
/// ```
macro_rules! impl_sum_of_fields {
    ($name:ident, $($field:ident),+) => {
        impl<I: GetSize> GetSize for $name<I> {
            fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut size = 0;

                $(
                    size += self.$field.get_heap_size_with_tracker(tracker);
                )+

                size
            }
        }
    };
}

impl_sum_of_fields!(Range, start, end);
impl_sum_of_fields!(RangeFrom, start);
impl_sum_of_fields!(RangeTo, end);
impl_sum_of_fields!(RangeToInclusive, end);

impl GetSize for RangeFull {}

impl<I: GetSize> GetSize for RangeInclusive<I> {
    #[inline]
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        (*self.start()).get_heap_size_with_tracker(tracker)
            + (*self.end()).get_heap_size_with_tracker(tracker)
    }
}

impl<T> GetSize for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        match self {
            Self::Borrowed(_borrowed) => 0,
            Self::Owned(owned) => owned.get_heap_size_with_tracker(tracker),
        }
    }
}

macro_rules! impl_size_set {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let size: usize = self
                    .iter()
                    .map(|elem| elem.get_heap_size_with_tracker(tracker))
                    .sum();

                let allocation_size = self.capacity() * T::get_stack_size();
                size + allocation_size
            }
        }
    };
}

macro_rules! impl_size_set_no_capacity {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                // We assume that values are held inside the heap.
                self.iter()
                    .map(|elem| elem.get_size_with_tracker(tracker))
                    .sum()
            }
        }
    };
}

impl_size_set_no_capacity!(BTreeSet);
impl_size_set!(BinaryHeap);
impl_size_set_no_capacity!(LinkedList);
impl_size_set!(VecDeque);

impl<K, V> GetSize for BTreeMap<K, V>
where
    K: GetSize,
    V: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        self.iter()
            .map(|(key, value)| {
                key.get_size_with_tracker(tracker) + value.get_size_with_tracker(tracker)
            })
            .sum()
    }
}

impl<K, V, S: ::std::hash::BuildHasher> GetSize for HashMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|(key, value)| {
                key.get_heap_size_with_tracker(tracker) + value.get_heap_size_with_tracker(tracker)
            })
            .sum();

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        size + allocation_size
    }
}

impl<T, S: ::std::hash::BuildHasher> GetSize for HashSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|elem| elem.get_heap_size_with_tracker(tracker))
            .sum();

        let allocation_size = self.capacity() * T::get_stack_size();
        size + allocation_size
    }
}

impl_size_set!(Vec);

macro_rules! impl_size_tuple {
    ($($t:ident, $T:ident),+) => {
        impl<$($T,)*> GetSize for ($($T,)*)
        where
            $(
                $T: GetSize,
            )*
        {
            fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
                let mut total = 0;

                let ($($t,)*) = self;
                $(
                    total += $t.get_heap_size_with_tracker(tracker);
                )*

                total
            }
        }
    }
}

macro_rules! execute_tuple_macro_16 {
    ($name:ident) => {
        $name!(v1, V1);
        $name!(v1, V1, v2, V2);
        $name!(v1, V1, v2, V2, v3, V3);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7);
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14, v15, V15
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14, v15, V15, v16, V16
        );
    };
}

execute_tuple_macro_16!(impl_size_tuple);

impl<T, const SIZE: usize> GetSize for [T; SIZE]
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // The array stack size already accounts for the stack size of the elements of the array.
        self.iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum()
    }
}

impl<T> GetSize for &[T] where T: GetSize {}

impl<T> GetSize for &T {}
impl<T> GetSize for &mut T {}
impl<T> GetSize for *const T {}
impl<T> GetSize for *mut T {}

impl<T> GetSize for Box<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        (**self).get_size_with_tracker(tracker)
    }
}

impl<T> GetSize for Rc<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(NonNull::from(Rc::as_ref(self)).cast()) {
            (**self).get_size_with_tracker(tracker)
        } else {
            0
        }
    }
}

impl<T> GetSize for RcWeak<T> {}

impl<T> GetSize for Arc<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if tracker.track(NonNull::from(Arc::as_ref(self)).cast()) {
            (**self).get_size_with_tracker(tracker)
        } else {
            0
        }
    }
}

impl<T> GetSize for ArcWeak<T> {}

impl<T> GetSize for Option<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        match self {
            None => 0,
            Some(value) => value.get_heap_size_with_tracker(tracker),
        }
    }
}

impl<T, E> GetSize for Result<T, E>
where
    T: GetSize,
    E: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // The results stack size already accounts for the values stack size.
        match self {
            Ok(value) => value.get_heap_size_with_tracker(tracker),
            Err(err) => err.get_heap_size_with_tracker(tracker),
        }
    }
}

impl<T> GetSize for Mutex<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // We assume that a `Mutex` holds its data at the stack.
        self.lock()
            .expect("Mutex is poisoned")
            .get_heap_size_with_tracker(tracker)
    }
}

impl<T> GetSize for RwLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // We assume that a `RwLock` holds its data at the stack.
        self.read()
            .expect("RwLock is poisoned")
            .get_heap_size_with_tracker(tracker)
    }
}

impl<T> GetSize for RefCell<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // We assume that a `RefCell` holds its data at the stack.
        // Use try_borrow to avoid panicking if the RefCell is already mutably borrowed.
        match self.try_borrow() {
            Ok(borrowed) => borrowed.get_heap_size_with_tracker(tracker),
            Err(_) => {
                // If the RefCell is already mutably borrowed, we cannot safely access it.
                // Return 0 for heap size to avoid panic, though this is a rare edge case.
                0
            }
        }
    }
}

impl<T> GetSize for OnceLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        // We assume that a `OnceLock` holds its data at the stack.
        match self.get() {
            None => 0,
            Some(value) => value.get_heap_size_with_tracker(tracker),
        }
    }
}

impl GetSize for String {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.capacity()
    }
}

impl GetSize for &str {}

impl GetSize for std::ffi::CString {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.as_bytes_with_nul().len()
    }
}

impl GetSize for &std::ffi::CStr {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.to_bytes_with_nul().len()
    }
}

impl GetSize for std::ffi::OsString {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl GetSize for &std::ffi::OsStr {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl GetSize for std::fs::DirBuilder {}
impl GetSize for std::fs::DirEntry {}
impl GetSize for std::fs::File {}
impl GetSize for std::fs::FileType {}
impl GetSize for std::fs::Metadata {}
impl GetSize for std::fs::OpenOptions {}
impl GetSize for std::fs::Permissions {}
impl GetSize for std::fs::ReadDir {}

impl<T> GetSize for std::io::BufReader<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        self.get_ref().get_heap_size_with_tracker(tracker) + self.capacity()
    }
}

impl<T> GetSize for std::io::BufWriter<T>
where
    T: GetSize + std::io::Write,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        self.get_ref().get_heap_size_with_tracker(tracker) + self.capacity()
    }
}

impl GetSize for std::path::PathBuf {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.capacity()
    }
}

impl GetSize for &std::path::Path {}

impl<T> GetSize for Box<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        let allocation_size = self.len() * T::get_stack_size();
        size + allocation_size
    }
}

impl GetSize for Box<str> {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl<T> GetSize for Rc<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        let allocation_size = self.len() * T::get_stack_size();
        size + allocation_size
    }
}

impl GetSize for Rc<str> {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

impl<T> GetSize for Arc<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        let allocation_size = self.len() * T::get_stack_size();
        size + allocation_size
    }
}

impl GetSize for Arc<str> {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

#[cfg(feature = "chrono")]
mod chrono {
    use crate::{GetSize, GetSizeTracker};

    impl GetSize for chrono::NaiveDate {}
    impl GetSize for chrono::NaiveTime {}
    impl GetSize for chrono::NaiveDateTime {}
    impl GetSize for chrono::Utc {}
    impl GetSize for chrono::FixedOffset {}
    impl GetSize for chrono::TimeDelta {}

    impl<Tz: chrono::TimeZone> GetSize for chrono::DateTime<Tz>
    where
        Tz::Offset: GetSize,
    {
        fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
            self.offset().get_heap_size_with_tracker(tracker)
        }
    }
}

#[cfg(feature = "chrono-tz")]
impl GetSize for chrono_tz::TzOffset {}

#[cfg(feature = "url")]
impl GetSize for url::Url {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.as_str().len()
    }
}

#[cfg(feature = "bytes")]
impl GetSize for bytes::Bytes {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

#[cfg(feature = "bytes")]
impl GetSize for bytes::BytesMut {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        self.len()
    }
}

#[cfg(feature = "hashbrown")]
impl<K, V, H> GetSize for hashbrown::HashMap<K, V, H>
where
    K: GetSize + Eq + std::hash::Hash,
    V: GetSize,
    H: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|(key, value)| {
                key.get_heap_size_with_tracker(tracker) + value.get_heap_size_with_tracker(tracker)
            })
            .sum();

        size + self.allocation_size()
    }
}

#[cfg(feature = "hashbrown")]
impl<T, H> GetSize for hashbrown::HashSet<T, H>
where
    T: GetSize + Eq + std::hash::Hash,
    H: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        size + self.allocation_size()
    }
}

#[cfg(feature = "hashbrown")]
impl<T> GetSize for hashbrown::HashTable<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        size + self.allocation_size()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> GetSize for smallvec::SmallVec<A>
where
    A::Item: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let mut size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        if self.len() > self.inline_size() {
            size += self.capacity() * <A::Item>::get_stack_size();
        }

        size
    }
}

#[cfg(feature = "thin-vec")]
impl<T> GetSize for thin_vec::ThinVec<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        if self.capacity() == 0 {
            // If it's the singleton we might not be a heap pointer.
            return 0;
        }

        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        let metadata_size = std::mem::size_of::<usize>() * 2; // Capacity and length.
        let allocation_size = self.capacity() * T::get_stack_size();
        size + metadata_size + allocation_size
    }
}

#[cfg(feature = "compact-str")]
impl GetSize for compact_str::CompactString {
    fn get_heap_size_with_tracker(&self, _tracker: &mut dyn GetSizeTracker) -> usize {
        if self.is_heap_allocated() {
            self.capacity()
        } else {
            0
        }
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S> GetSize for indexmap::IndexMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
    S: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|(key, value)| {
                key.get_heap_size_with_tracker(tracker) + value.get_heap_size_with_tracker(tracker)
            })
            .sum();

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        size + allocation_size
    }
}

#[cfg(feature = "indexmap")]
impl<T, S> GetSize for indexmap::IndexSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        let allocation_size = self.capacity() * T::get_stack_size();
        size + allocation_size
    }
}

#[cfg(feature = "ordermap")]
impl<K, V, S> GetSize for ordermap::OrderMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
    S: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|(key, value)| {
                key.get_heap_size_with_tracker(tracker) + value.get_heap_size_with_tracker(tracker)
            })
            .sum();

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        size + allocation_size
    }
}

#[cfg(feature = "ordermap")]
impl<T, S> GetSize for ordermap::OrderSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker(&self, tracker: &mut dyn GetSizeTracker) -> usize {
        let size: usize = self
            .iter()
            .map(|element| element.get_heap_size_with_tracker(tracker))
            .sum();

        let allocation_size = self.capacity() * T::get_stack_size();
        size + allocation_size
    }
}
