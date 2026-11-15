use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::{GetSize, GetSizeTracker};

/// Helper macro used to sum the heap sizes of specific fields.
macro_rules! impl_sum_of_fields {
    ($name:ident, $($field:ident),+) => {
        impl<I: GetSize> GetSize for $name<I> {
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
                let size = 0;

                // Rebinding instead of assigning keeps the variables immutable, however many
                // fields the macro is expanded for.
                $(
                    let (elem_size, tracker) = self.$field.get_heap_size_with_tracker(tracker);
                    let size = size + elem_size;
                )+

                (size, tracker)
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
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (start_size, tracker) = (*self.start()).get_heap_size_with_tracker(tracker);
        let (end_size, tracker) = (*self.end()).get_heap_size_with_tracker(tracker);
        (start_size + end_size, tracker)
    }
}
