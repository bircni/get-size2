use std::ffi::{OsStr, OsString};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use crate::{GetSize, GetSizeTracker};

impl GetSize for OsString {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl GetSize for PathBuf {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.capacity(), tracker)
    }
}

// Borrowed data belongs to whoever owns it, so these report a heap size of zero.
impl GetSize for &OsStr {}
impl GetSize for &Path {}

impl GetSize for std::fs::DirBuilder {}
impl GetSize for std::fs::DirEntry {}
impl GetSize for std::fs::File {}
impl GetSize for std::fs::FileType {}
impl GetSize for std::fs::Metadata {}
impl GetSize for std::fs::OpenOptions {}
impl GetSize for std::fs::Permissions {}
impl GetSize for std::fs::ReadDir {}

impl GetSize for Instant {}
impl GetSize for SystemTime {}

impl<T> GetSize for BufReader<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (total, tracker) = T::get_heap_size_with_tracker(self.get_ref(), tracker);
        (total + self.capacity(), tracker)
    }
}

impl<T> GetSize for BufWriter<T>
where
    T: GetSize + Write,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (total, tracker) = T::get_heap_size_with_tracker(self.get_ref(), tracker);
        (total + self.capacity(), tracker)
    }
}
