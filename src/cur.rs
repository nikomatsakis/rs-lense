
use std::{marker, mem, ops, slice};

use {Primitive, Result, Error};

/// Cursor state for seeking into [u8] slices
pub struct Cursor<S> {
    cur:   *mut u8,
    end:   *mut u8,
    waste: usize,
    marker: marker::PhantomData<*mut S>,
}

impl<S> ops::Deref for Cursor<S> where S: ops::Deref<Target=[u8]> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.cur, self.end as usize - self.cur as usize)
        }
    }
}

impl<S> ops::DerefMut for Cursor<S> where S: ops::Deref<Target=[u8]> + ops::DerefMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.cur, self.end as usize - self.cur as usize)
        }
    }
}

impl<S> Cursor<S> where S: ops::Deref<Target=[u8]> {
    /// Start maintaining a cursor over some type that derefrences to [u8]
    pub fn new(s: S) -> Self {
        let p = s.as_ptr() as *mut u8;
        Cursor {
            cur: p,
            end: unsafe { p.offset(s.len() as isize) },
            waste: 0,
            marker: marker::PhantomData,
        }
    }

    /// Advance the cursor n-bytes, and return a new cursor over that segment
    pub fn advance(&mut self, n: u64) -> Result<Self> {
        let p = unsafe { self.cur.offset(n as isize) };
        if self.end >= p {
            Ok(Cursor {
                cur: mem::replace(&mut self.cur, p),
                end: p,
                waste: 0,
                marker: self.marker,
            })
        } else {
            Err(Error::ExpectedBytes(n as usize))
        }
    }

    /// Return the number of bytes wasted due to padding
    pub fn waste(&self) -> usize {
        self.waste
    }

    /// Return the number of bytes remaining
    pub fn remaining(&self) -> usize {
        self.end as usize - self.cur as usize
    }

    #[cfg(feature = "no_automatic_padding")]
    pub fn align<T>(&mut self) -> Result<()> { Ok(()) }

    #[cfg(not(feature = "no_automatic_padding"))]
    pub fn align<T>(&mut self) -> Result<()> {
        let pad = self.cur as usize % mem::align_of::<T>();
        if pad > 0 {
            self.advance((mem::align_of::<T>() - pad) as u64).map(|_| self.waste += pad)
        } else {
            Ok(())
        }
    }

    pub fn dice<T: Primitive>(&mut self) -> Result<RefMut<T, S>> {
        try!(self.align::<T>());
        self.advance(mem::size_of::<T>() as u64).map(RefMut::new)
    }
}

pub struct RefMut<T, S> {
    data: *mut T,
    marker: marker::PhantomData<*mut S>,
}

impl<T, S> RefMut<T, S> {
    pub fn new(c: Cursor<S>) -> Self where S: ops::Deref<Target=[u8]> {
        RefMut {
            data: c.as_ptr() as *mut T,
            marker: marker::PhantomData,
        }
    }
}

impl<T, S> ops::Deref for RefMut<T, S> where S: ops::Deref {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T, S> ops::DerefMut for RefMut<T, S> where S: ops::Deref + ops::DerefMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}
