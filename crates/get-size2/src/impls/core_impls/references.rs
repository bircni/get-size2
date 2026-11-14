use core::pin::Pin;
use core::ptr::NonNull;
#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic::AtomicPtr;

use crate::GetSize;

// A reference only borrows its data, which belongs to whoever owns it, so all of these report a
// heap size of zero. The `?Sized` bound covers unsized targets as well, e.g. `&[T]`, `&str` and
// `&dyn Trait`.

impl<T: ?Sized> GetSize for &T {}
impl<T: ?Sized> GetSize for &mut T {}
impl<T: ?Sized> GetSize for *const T {}
impl<T: ?Sized> GetSize for *mut T {}
impl<T: ?Sized> GetSize for NonNull<T> {}
#[cfg(target_has_atomic = "ptr")]
impl<T> GetSize for AtomicPtr<T> {}

// Pinning does not change who owns the data.
impl<T: ?Sized> GetSize for Pin<&T> {}
impl<T: ?Sized> GetSize for Pin<&mut T> {}
