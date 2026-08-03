use crate::GetSize;

// A reference only borrows its data, which belongs to whoever owns it, so all of these report a
// heap size of zero. The `?Sized` bound covers unsized targets as well, e.g. `&[T]`, `&str` and
// `&dyn Trait`.

impl<T: ?Sized> GetSize for &T {}
impl<T: ?Sized> GetSize for &mut T {}
impl<T: ?Sized> GetSize for *const T {}
impl<T: ?Sized> GetSize for *mut T {}
