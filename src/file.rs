use std::fs::File;
use std::io::{self, Read};
use std::collections::HashMap;

use {SeekablePool, Lense};

enum PoolPolicy {
    Strict, // Do not allocate more memory when the pool runs out of storage.
}

#[derive(Clone)]
enum CacheEntry {
    Locked(usize),   // A mutable lense is active
    Readonly(usize), // An immutable lense is active
    Unlocked(usize), // An entry exists and is not in use
}

/// A cached file access lense. Suitable for databases.
pub struct LenseFile<L: Lense> {
    file: Option<File>,
    pool: SeekablePool<L>,
    cache: HashMap<usize, CacheEntry>,
    policy: PoolPolicy,
}

impl<L> LenseFile<L> where L: Lense {
    /// Allocate the pool and cache such that we can store a `cap` of type L.
    pub fn with_capacity(cap: usize) -> Self {
        LenseFile {
            file: None,
            pool: SeekablePool::with_capacity(cap),
            cache: HashMap::with_capacity(cap),
            policy: PoolPolicy::Strict,
        }
    }

    /// Read a file directly into the pool.
    pub fn read_file(&mut self, file: &mut File) -> io::Result<usize> {
        match self.policy {
            PoolPolicy::Strict => file.read(&mut *self),
        }
    }

// Lock when leasing lenses.
// Lenses may update the disk state.
// An unlocked entry can be freely updated without a read first.
// An entry may be replaced with another - write new, return old.
// Appending to the statefile is cheap - seek to the end, write.
//
// Overwriting pool and cached data can only occur in positions that are not
// locked.  If the pool runs out of allocated storage and no positions are
// unlocked, then either allocate a second pool or complain to the consumer that
// too many entries are currently active. (Depending on policy)
//
// The entire state may be maintained in ram with disk writes occurring only on
// snapshot requests.
//
// pool_policy    // Can the pool allocate more ram when it has no unused slots
//                // and is requested for an entry in the persistant store.
//
// with_pool_size // Configure ram backing pool size
// cache_policy   // Configure cache policy: Frequency, Sequence (read-a-head)
//                // Frequency: Store entries frequently requested (weighted)
//                // Sequence: Expecting sequential reads. N, N+1 .. M
//
// snapshot       // Save the current state to another file
//
// [Entry functions] // pool may also implement these for quick snapshot
// management.
//
// update_cache // Update ram value
// update_store // Update persistant value
//
// flush // Push cache to persistant storage
// reset // Ignore cache, recover persistant values

}

impl<L: Lense> ::std::ops::Deref for LenseFile<L> {
    type Target = SeekablePool<L>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl<L: Lense> ::std::ops::DerefMut for LenseFile<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool
    }
}
