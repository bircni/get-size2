#![allow(clippy::allow_attributes, reason = "needed if features are enabled")]
#[allow(unused_imports, reason = "needed if features are enabled")]
use crate::{GetSize, GetSizeTracker};

#[cfg(feature = "chrono")]
impl GetSize for chrono::NaiveDate {}
#[cfg(feature = "chrono")]
impl GetSize for chrono::NaiveTime {}
#[cfg(feature = "chrono")]
impl GetSize for chrono::NaiveDateTime {}
#[cfg(feature = "chrono")]
impl GetSize for chrono::Utc {}
#[cfg(feature = "chrono")]
impl GetSize for chrono::FixedOffset {}
#[cfg(feature = "chrono")]
impl GetSize for chrono::TimeDelta {}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> GetSize for chrono::DateTime<Tz>
where
    Tz::Offset: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        <Tz::Offset>::get_heap_size_with_tracker(self.offset(), tracker)
    }
}

#[cfg(feature = "chrono-tz")]
impl GetSize for chrono_tz::TzOffset {}

#[cfg(feature = "url")]
impl GetSize for url::Url {
    fn get_heap_size_with_tracker<T: crate::GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.as_str().len(), tracker)
    }
}

#[cfg(feature = "bytes")]
impl GetSize for bytes::Bytes {
    fn get_heap_size_with_tracker<T: crate::GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

#[cfg(feature = "bytes")]
impl GetSize for bytes::BytesMut {
    fn get_heap_size_with_tracker<T: crate::GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

#[cfg(feature = "hashbrown")]
impl<K, V, H> GetSize for hashbrown::HashMap<K, V, H>
where
    K: GetSize + Eq + std::hash::Hash,
    V: GetSize,
    H: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        (size + self.allocation_size(), tracker)
    }
}

#[cfg(feature = "hashbrown")]
impl<T, H> GetSize for hashbrown::HashSet<T, H>
where
    T: GetSize + Eq + std::hash::Hash,
    H: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + self.allocation_size(), tracker)
    }
}

#[cfg(feature = "hashbrown")]
impl<T> GetSize for hashbrown::HashTable<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + self.allocation_size(), tracker)
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> GetSize for smallvec::SmallVec<A>
where
    A::Item: GetSize,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (mut size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = <A::Item>::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        if self.len() > self.inline_size() {
            size += self.capacity() * <A::Item>::get_stack_size();
        }

        (size, tracker)
    }
}

#[cfg(feature = "thin-vec")]
impl<T> GetSize for thin_vec::ThinVec<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        if self.capacity() == 0 {
            // If it's the singleton we might not be a heap pointer.
            return (0, tracker);
        }

        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let metadata_size = std::mem::size_of::<usize>() * 2; // Capacity and length.
        let allocation_size = self.capacity() * T::get_stack_size();
        (size + metadata_size + allocation_size, tracker)
    }
}

#[cfg(feature = "compact-str")]
impl GetSize for compact_str::CompactString {
    fn get_heap_size_with_tracker<T: crate::GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        let size = if self.is_heap_allocated() {
            self.capacity()
        } else {
            0
        };

        (size, tracker)
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S> GetSize for indexmap::IndexMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
    S: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        (size + allocation_size, tracker)
    }
}

#[cfg(feature = "indexmap")]
impl<T, S> GetSize for indexmap::IndexSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.capacity() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}

#[cfg(feature = "ordermap")]
impl<K, V, S> GetSize for ordermap::OrderMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
    S: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        (size + allocation_size, tracker)
    }
}

#[cfg(feature = "ordermap")]
impl<T, S> GetSize for ordermap::OrderSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: crate::GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.capacity() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}
