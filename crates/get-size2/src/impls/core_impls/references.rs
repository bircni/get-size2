use core::ffi::CStr;

use crate::GetSize;

// A reference only borrows its data, which belongs to whoever owns it, so all of these report a
// heap size of zero.

impl<T> GetSize for &[T] where T: GetSize {}

impl<T> GetSize for &T {}
impl<T> GetSize for &mut T {}
impl<T> GetSize for *const T {}
impl<T> GetSize for *mut T {}

impl GetSize for &str {}
impl GetSize for &CStr {}
