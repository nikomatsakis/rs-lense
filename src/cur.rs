use std::{marker, mem, ops, slice};

use {Endian, Error, Mut, Ref, Result};
use tag::{BoolGuard, LenseTag};

/// Cursor state for seeking into [u8] slices
pub struct Cursor<S> {
    cur:   *mut u8,
    end:   *mut u8,
    waste: usize,
    tag:   Option<LenseTag<S>>,
    m: marker::PhantomData<*mut S>,
}

impl<S> ops::Deref for Cursor<S> where S: Ref {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.cur, self.end as usize - self.cur as usize)
        }
    }
}

impl<S> ops::DerefMut for Cursor<S> where S: Mut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.cur, self.end as usize - self.cur as usize)
        }
    }
}

impl<S> Cursor<S> where S: Ref {
    /// Start maintaining a cursor over some type that derefrences to [u8]
    pub fn new(s: S) -> Self {
        let p = s.as_ptr() as *mut u8;
        Cursor {
            cur: p,
            end: unsafe { p.offset(s.len() as isize) },
            waste: 0,
            tag: None,
            m: marker::PhantomData,
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
                tag: self.tag.clone(),
                m: self.m,
            })
        } else {
            Err(Error::ExpectedBytes(n as usize))
        }
    }

    pub fn set_tag(&mut self, t: LenseTag<S>) {
        self.tag = Some(t);
    }

    pub fn tag_bits(&mut self, n: usize) -> Result<usize> {
        match self.tag {
            Some(ref mut t) => t.bits(n),
            None => Err(Error::NoTagDefined),
        }
    }

    /// Return a boolean bit from the tag if present
    pub fn tag_bit(&mut self) -> Result<BoolGuard<S>> {
        match self.tag {
            Some(ref mut t) => t.bit(),
            None => Err(Error::NoTagDefined),
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

    pub fn dice<T>(&mut self) -> Result<RefMut<T, S>> where T: Endian {
        try!(self.align::<T>());
        self.advance(mem::size_of::<T>() as u64).map(RefMut::new)
    }
}

pub struct RefMut<T, S> {
    data: *mut T,
    m: marker::PhantomData<*mut S>,
}

// Todo endianness on T
impl<T, S> RefMut<T, S>
    where T: Endian, S: Ref
{
    pub fn new(c: Cursor<S>) -> Self {
        RefMut {
            data: c.as_ptr() as *mut T,
            m: marker::PhantomData,
        }
    }

    pub fn get(&self) -> T where T: Copy {
        Endian::handle(unsafe { *self.data })
    }

    pub fn set(&mut self, t: T) where S: Mut {
        unsafe { *self.data = Endian::handle(t) }
    }
}
