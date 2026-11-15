#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::collections::BTreeSet;
#[cfg(feature = "std")]
use core::hash::{BuildHasherDefault, Hasher};
#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(feature = "std")]
use std::sync::{Arc, Mutex, RwLock};

/// A tracker which makes sure that shared ownership objects are only accounted for once.
pub trait GetSizeTracker {
    /// Tracks an arbitrary object located at `addr`.
    ///
    /// Returns `true` if the reference, as indexed by the pointed to `addr`, has not yet
    /// been seen by this tracker. Otherwise it returns `false`.
    ///
    /// The target may be unsized, like a `str` or a slice. Only the address identifies the
    /// object then, the length or vtable carried by such a pointer is not part of its identity.
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool;
}

impl<T: GetSizeTracker> GetSizeTracker for &mut T {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(*self, addr)
    }
}

#[cfg(feature = "alloc")]
impl<T: GetSizeTracker> GetSizeTracker for Box<T> {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(&mut **self, addr)
    }
}

#[cfg(feature = "std")]
impl<T: GetSizeTracker> GetSizeTracker for Mutex<T> {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        let tracker = self
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

#[cfg(feature = "std")]
impl<T: GetSizeTracker> GetSizeTracker for RwLock<T> {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        // The exclusive borrow already guarantees unique access, so no locking is needed.
        let tracker = self
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

#[cfg(feature = "std")]
impl<T: GetSizeTracker> GetSizeTracker for Arc<Mutex<T>> {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

#[cfg(feature = "std")]
impl<T: GetSizeTracker> GetSizeTracker for Arc<RwLock<T>> {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

/// Hashes the addresses seen by a [`StandardTracker`].
///
/// The default `SipHash` of the standard library is built to withstand attacker chosen keys,
/// which is wasted effort for the addresses of the measured value's own allocations. A single
/// multiplication mixes them well enough and is noticeably cheaper.
#[cfg(feature = "std")]
#[derive(Debug, Default, Clone, Copy)]
struct AddressHasher(u64);

#[cfg(feature = "std")]
impl AddressHasher {
    // The 64 bit golden ratio constant, as used by Fibonacci hashing.
    const MULTIPLIER: u64 = 0x9E37_79B9_7F4A_7C15;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the product is split into its two halves on purpose"
    )]
    const fn mix(&mut self, value: u64) {
        // Allocations are aligned, so the lowest bits of an address are always zero and stay
        // zero in the low half of a product, while its high half is mixed well. A hash table
        // takes the bucket from the low bits of the hash and a tag from its top bits, on a 32
        // bit target both from the low half. Folding the two halves of the full 128 bit
        // product onto each other leaves no weak bit at either end.
        let product = (self.0 ^ value) as u128 * Self::MULTIPLIER as u128;

        self.0 = (product as u64) ^ ((product >> 64) as u64);
    }
}

#[cfg(feature = "std")]
impl Hasher for AddressHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.mix(u64::from(byte));
        }
    }

    fn write_usize(&mut self, value: usize) {
        // A `usize` is at most 64 bits wide on every supported target.
        self.mix(value as u64);
    }
}

// The set of addresses already seen by a `StandardTracker`. A `HashSet` is only provided by
// `std`, so `no_std` builds fall back to the `alloc` only `BTreeSet`.
#[cfg(feature = "std")]
type SeenAddresses = HashSet<usize, BuildHasherDefault<AddressHasher>>;
#[cfg(all(feature = "alloc", not(feature = "std")))]
type SeenAddresses = BTreeSet<usize>;

/// A simple standard tracker which can be used to track shared ownership references.
///
/// It remembers the address of every allocation it has been asked to track. Creating one does
/// not allocate, so starting a fresh tracker for a single measurement is cheap.
#[cfg(feature = "alloc")]
#[derive(Debug, Default)]
pub struct StandardTracker {
    inner: SeenAddresses,
}

#[cfg(feature = "alloc")]
impl StandardTracker {
    /// Creates a new tracker which has not seen any address yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets all addresses seen so far, keeping the allocated memory for reuse.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[cfg(feature = "alloc")]
impl GetSizeTracker for StandardTracker {
    fn track<A: ?Sized>(&mut self, addr: *const A) -> bool {
        self.inner.insert(addr.addr())
    }
}

/// A pseudo tracker which does not track anything.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoTracker {
    answer: bool,
}

impl NoTracker {
    /// Creates a new pseudo tracker, which will always return the given `answer`.
    #[must_use]
    pub const fn new(answer: bool) -> Self {
        Self { answer }
    }

    /// Get the answer which will always be returned by this pseudo tracker.
    #[must_use]
    pub const fn answer(&self) -> bool {
        self.answer
    }

    /// Changes the answer which will always be returned by this pseudo tracker.
    pub const fn set_answer(&mut self, answer: bool) {
        self.answer = answer;
    }
}

impl GetSizeTracker for NoTracker {
    fn track<A: ?Sized>(&mut self, _addr: *const A) -> bool {
        self.answer
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use core::hash::Hasher;
    use std::collections::HashSet;

    use super::AddressHasher;

    fn hash(address: usize) -> u64 {
        let mut hasher = AddressHasher::default();
        hasher.write_usize(address);
        hasher.finish()
    }

    #[test]
    fn address_hasher_spreads_aligned_addresses() {
        const COUNT: usize = 1 << 16;

        // Allocations are aligned, so their addresses only differ above the lowest bits. A hash
        // table selects the bucket with the low bits of the hash, the more of them the larger it
        // grows, and tags the entry with seven bits from the top: of the `u64` on a 64 bit
        // target, and of its lower half on a 32 bit one. All of them have to vary for the
        // addresses of neighboring allocations.
        for (stride, base) in [
            (8, 0),
            (16, 0x2000_0000),
            (32, 0x1234_0000),
            (48, 0x5555_0000),
            (64, 0x1_0000),
            (4096, 0x7000_0000),
        ] {
            let hashes: std::vec::Vec<u64> = (0..COUNT)
                .map(|index| hash(base + index * stride))
                .collect();
            let distinct = |part: fn(u64) -> u64| {
                hashes
                    .iter()
                    .map(|hash| part(*hash))
                    .collect::<HashSet<_>>()
                    .len()
            };

            // A random function fills about 41 300 of the 65 536 buckets.
            let buckets = distinct(|hash| hash & 0xFFFF);
            assert!(buckets > 36_000, "stride {stride}: {buckets} buckets");

            // The bits a table of many millions of entries adds to its bucket index.
            let upper_buckets = distinct(|hash| (hash >> 20) & 0xFFF);
            assert!(upper_buckets > 4000, "stride {stride}: {upper_buckets}");

            assert_eq!(distinct(|hash| hash >> 57), 128, "stride {stride}: tags");
            assert_eq!(
                distinct(|hash| (hash & 0xFFFF_FFFF) >> 25),
                128,
                "stride {stride}: tags on a 32 bit target"
            );

            // No bit may be the same for every address.
            let always_set = hashes.iter().fold(u64::MAX, |bits, hash| bits & hash);
            let ever_set = hashes.iter().fold(0, |bits, hash| bits | hash);
            assert_eq!(always_set, 0, "stride {stride}");
            assert_eq!(ever_set, u64::MAX, "stride {stride}");
        }
    }
}
