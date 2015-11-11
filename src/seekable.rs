use std::cell::Cell;

use {Lense, SliceRef, SliceMut, Dice};
use aligned::Aligned;

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

impl<'a, L: Lense> SeekablePool<L> {
    pub fn with_capacity(cap: usize) -> Self {
        SeekablePool {
            pool: vec![0u64; div_up(cap * L::size(), 8)],
            state: vec![Cell::new(false); cap],
            marker: ::std::marker::PhantomData,
        }
    }

    fn lense(&self, pos: usize) -> Option<Guard<L::Ref>> where L: SliceRef<'a> {
        match self.state.get(pos) {
            Some(ref mut lock) if !lock.get() => {
                let ref mut ptr = Aligned::new(unsafe { // &self[L::size() * pos .. L::size()]
                    ::std::slice::from_raw_parts(
                        (self.pool.as_ptr() as *const u8).offset((L::size() * pos) as isize),
                        L::size())
                });

                lock.set(true);

                Some(Guard(&lock, L::slice(ptr)))
            }
            Some(..) => None,
            None => panic!("Invalid index! {}", pos),
        }
    }

    fn lense_mut(&self, pos: usize) -> Option<Guard<L::Mut>> where L: SliceMut<'a> {
        match self.state.get(pos) {
            Some(ref mut lock) if !lock.get() => {
                let ref mut ptr = Aligned::new(unsafe { // &mut self[L::size() * pos .. L::size()]
                    ::std::slice::from_raw_parts_mut(
                        (self.pool.as_ptr() as *mut u8).offset((L::size() * pos) as isize),
                        L::size())
                });

                lock.set(true);

                Some(Guard(&lock, L::slice_mut(ptr)))
            }
            Some(..) => None,
            None => panic!("Invalid index! {}", pos),
        }
    }

    pub fn iter(&'a self) -> IterRef<L> where L: SliceRef<'a> {
        IterRef { pool: self, cur: 0 }
    }

    pub fn iter_mut(&'a mut self) -> IterMut<'a, L> where L: SliceMut<'a> {
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

pub struct Guard<'a, T>(&'a Cell<bool>, T);

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl<'a, T> ::std::ops::Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<'a, T> ::std::ops::DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

// Should iterators be reserved for lense_vector?

pub struct IterRef<'a, L: 'a + SliceRef<'a>> {
    pool: &'a SeekablePool<L>,
    cur: usize,
}

impl<'a, L: SliceRef<'a>> Iterator for IterRef<'a, L> {
    type Item = Guard<'a, L::Ref>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.pool.state.len() {
            let ret = self.pool.lense(self.cur);
            self.cur += 1;
            ret
        } else { None }
    }
}

impl<'a, L: SliceRef<'a>> ExactSizeIterator for IterRef<'a, L> {
    fn len(&self) -> usize {
        self.pool.state.capacity()
    }
}

pub struct IterMut<'a, L: 'a + SliceMut<'a>> {
    pool: &'a SeekablePool<L>,
    cur: usize,
}

impl<'a, L: SliceMut<'a>> Iterator for IterMut<'a, L> {
    type Item = Guard<'a, L::Mut>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.pool.state.len() {
            let ret = self.pool.lense_mut(self.cur);
            self.cur += 1;
            ret
        } else { None }
    }
}

impl<'a, L: SliceMut<'a>> ExactSizeIterator for IterMut<'a, L> {
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
