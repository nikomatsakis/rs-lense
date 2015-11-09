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

pub struct LenseFile<L: Lense> {
    file: File,
    pool: SeekablePool<L>,
    cache: HashMap<usize, CacheEntry>,
    policy: PoolPolicy,
}

impl<L> LenseFile<L> where L: Lense {
    pub fn from_file(file: File, cap: usize) -> Self {
        LenseFile {
            file: file,
            pool: SeekablePool::with_capacity(cap),
            cache: HashMap::with_capacity(cap),
            policy: PoolPolicy::Strict,
        }
    }

    pub fn init(&mut self) -> io::Result<usize> {
        match self.policy {
            PoolPolicy::Strict => self.file.read(&mut *self.pool),
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

#[cfg(test)]
mod test{
    use std::fs::File;
    use super::*;
    use Lense;

    mk_lense_ty!{struct Alice ref
        a:  u8,        // a
        bc: (u8, u16), // b c
        dg: [u8; 4],   // d e f g
        h:  u64,       // h
    }

    // ~ $ hexdump -C lense-testing-file.dat
    // 00000000  00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f
    // *
    // 00000050
    //           a. b. c. .. d. e. f. g.  h. .. .. .. .. .. .. ..

    #[test]
    fn read_lense_file() {
        let mut f = LenseFile::<Alice>::from_file(File::open("lense-testing-file.dat").unwrap(), 5);

        assert_eq!(f.init().unwrap(), Alice::size() * 5);

        for guard in f.iter() {
            let Alice { a, bc: (b, c), dg: [d, e, f, g], h } = *guard;
            assert_eq!(*a, 0);
            assert_eq!(*b, 1);
            assert_eq!(*c, 0x0302);
            assert_eq!(*d, 4);
            assert_eq!(*e, 5);
            assert_eq!(*f, 6);
            assert_eq!(*g, 7);
            assert_eq!(*h, 0xF0E0D0C0B0A0908);
        }
    }
}
