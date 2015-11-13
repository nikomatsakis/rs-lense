use std::cell::Cell;

use {Lense, LenseMut, Mode, IsRef, IsMut};
use aligned::Aligned;

/// A 8-byte aligned random access backing collection supporting locking on borrows to prevent
/// aliasing.
pub struct SeekablePool<L: Lense> {
    // Backing u64 pool
    pool: Vec<u64>,
    // Lock state
    state: Vec<Cell<bool>>,
    // The pool is optimized for this type
    marker: ::std::marker::PhantomData<*const L>,
}

fn div_up(n: usize, m: usize) -> usize {
    if n % m == 0 {
        n / m
    } else {
        n / m + 1
    }
}

impl<L: Lense> SeekablePool<L> {
    /// Prepare a collection to store `cap` of type L
    pub fn with_capacity(cap: usize) -> Self {
        SeekablePool {
            pool: vec![0u64; div_up(cap * L::size(), 8)],
            state: vec![Cell::new(false); cap],
            marker: ::std::marker::PhantomData,
        }
    }

    fn lense(&self, pos: usize) -> Option<Guard<<L as Mode<IsRef>>::Return>> {
        match self.state.get(pos) {
            Some(ref mut lock) if !lock.get() => {
                let ref mut ptr = Aligned::new(unsafe { // &self[L::size() * pos .. L::size()]
                    ::std::slice::from_raw_parts(
                        (self.pool.as_ptr() as *const u8).offset((L::size() * pos) as isize),
                        L::size())
                });

                lock.set(true);

                Some(Guard { lock: &lock, value: L::lense(ptr) })
            }
            Some(..) => None,
            None => panic!("Invalid index! {}", pos),
        }
    }

    fn lense_mut(&self, pos: usize) -> Option<Guard<<L as Mode<IsMut>>::Return>> where L: LenseMut {
        match self.state.get(pos) {
            Some(ref mut lock) if !lock.get() => {
                let ref mut ptr = Aligned::new(unsafe { // &mut self[L::size() * pos .. L::size()]
                    ::std::slice::from_raw_parts_mut(
                        (self.pool.as_ptr() as *mut u8).offset((L::size() * pos) as isize),
                        L::size())
                });

                lock.set(true);

                Some(Guard { lock: &lock, value: L::lense_mut(ptr) })
            }
            Some(..) => None,
            None => panic!("Invalid index! {}", pos),
        }
    }

    /// Iterate immutably over the pool's collection of lenses.
    pub fn iter(&self) -> IterRef<L> {
        IterRef { pool: self, cur: 0 }
    }

    /// Iterate mutably over the pool's collection of lenses.
    pub fn iter_mut(&mut self) -> IterMut<L> where L: LenseMut {
        IterMut { pool: self, cur: 0 }
    }
}

impl<L: Lense> ::std::ops::Deref for SeekablePool<L> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { // Vec<u64> -> &[u8]
            ::std::slice::from_raw_parts(self.pool.as_ptr() as *const u8,
                                         self.state.capacity() * L::size())
        }
    }
}

impl<L: Lense> ::std::ops::DerefMut for SeekablePool<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { // Vec<u64> -> &mut [u8]
            ::std::slice::from_raw_parts_mut(self.pool.as_mut_ptr() as *mut u8,
                                             self.state.capacity() * L::size())
        }
    }
}

// Guard the lense until it is dropped and then release the lock on the pool position

pub struct Guard<'a, T> {
    lock: &'a Cell<bool>,
    value: T
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.set(false);
    }
}

impl<'a, T> ::std::ops::Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T> ::std::ops::DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

// Should iterators be reserved for lense_vector?

/// Iterate immutably over the pool's collection of lenses.
pub struct IterRef<'a, L: 'a + Lense> {
    pool: &'a SeekablePool<L>,
    cur: usize,
}

impl<'a, L: Lense> Iterator for IterRef<'a, L> {
    type Item = Guard<'a, <L as Mode<IsRef>>::Return>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.pool.state.len() {
            let ret = self.pool.lense(self.cur);
            self.cur += 1;
            ret
        } else { None }
    }
}

impl<'a, L: Lense> ExactSizeIterator for IterRef<'a, L> {
    fn len(&self) -> usize {
        self.pool.state.capacity()
    }
}

/// Iterate mutably over the pool's collection of lenses.
pub struct IterMut<'a, L: 'a + LenseMut> {
    pool: &'a SeekablePool<L>,
    cur: usize,
}

impl<'a, L: LenseMut> Iterator for IterMut<'a, L> {
    type Item = Guard<'a, <L as Mode<IsMut>>::Return>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.pool.state.len() {
            let ret = self.pool.lense_mut(self.cur);
            self.cur += 1;
            ret
        } else { None }
    }
}

impl<'a, L: LenseMut> ExactSizeIterator for IterMut<'a, L> {
    fn len(&self) -> usize {
        self.pool.state.capacity()
    }
}

#[cfg(test)]
mod test {
    use SeekablePool;

    #[test]
    fn locking() {
        let pool = SeekablePool::<u8>::with_capacity(1);
        if let Some(_guard) = pool.lense(0) {
            assert!(pool.lense(0).is_none());
        } else { unreachable!() }
        assert!(pool.lense(0).is_some());
    }
}
