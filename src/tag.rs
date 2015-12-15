use std::{marker, mem, result};
use {Cursor, Error, Lense, Mut, Primitive, Ref, Result};

/// Set the lense tag for the cursor!
pub struct Tag<T: Primitive>(marker::PhantomData<T>);

unsafe impl<T, S> Lense<S> for Tag<T> where T: Primitive, S: Ref {
    type Ret = ();

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        let tag = try!(c.advance(mem::size_of::<T>() as u64).map(LenseTag::new));
        c.set_tag(tag);
        Ok(())
    }
}

/// Compact tag for booleans, unions, and enums
pub struct LenseTag<S> {
    cur: *mut u8,
    end: *mut u8,
    n: usize,
    m: marker::PhantomData<S>,
}

// Manual impl Clone due to PhantomData possibly being &mut [u8]
impl<S> Clone for LenseTag<S> {
    fn clone(&self) -> Self {
        LenseTag { .. *self }
    }
}

// TODO: Alignment for bits(n)
impl<S> LenseTag<S> where S: Ref {
    fn new(c: Cursor<S>) -> Self {
        LenseTag {
            cur: c.as_ptr() as *mut u8,
            end: unsafe {
                c.as_ptr().offset(c.len() as isize) as *mut u8
            },
            n: 0,
            m: marker::PhantomData,
        }
    }

    /// Request n-bits from the tag as usize
    pub fn bits(&mut self, n: usize) -> Result<usize> {
        if self.n == 8 {
            self.n = 0;
            self.cur = unsafe { self.cur.offset(1) };
        }

        if self.cur < self.end {
            let m = (n.next_power_of_two() - 1) << self.n;
            self.n += 1;
            unsafe {
                Ok((*self.cur as usize) & m)
            }
        } else {
            Err(Error::ExpectedBits(n))
        }
    }

    /// Request a boolean from the tag
    pub fn bit(&mut self) -> Result<BoolGuard<S>> {
        if self.n == 8 {
            self.n = 0;
            self.cur = unsafe { self.cur.offset(1) };
        }

        if self.cur < self.end {
            let m = 1 << self.n;
            self.n += 1;
            Ok(BoolGuard::new(self.cur, m))
        } else {
            Err(Error::ExpectedBits(1))
        }
    }
}


/// Union lense option
unsafe impl<T, S> Lense<S> for Option<T> where T: Lense<S>, S: Ref {
    type Ret = Option<T::Ret>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        if try!(bool::lense(c)).get() {
            Ok(Some(try!(T::lense(c))))
        } else {
            Ok(None)
        }
    }
}

/// Union lense result
unsafe impl<L, R, S> Lense<S> for result::Result<L, R>
    where L: Lense<S>,
          R: Lense<S>,
          S: Ref
{
    type Ret = result::Result<L::Ret, R::Ret>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        if try!(bool::lense(c)).get() {
            Ok(Ok(try!(L::lense(c))))
        } else {
            Ok(Err(try!(R::lense(c))))
        }
    }
}

pub mod padded {
    use std::{marker, option, result};
    use {Cursor, Lense, Ref, Result as Res, SizedLense};

    pub struct Option<T>(marker::PhantomData<T>);
    pub struct Result<T, F>(marker::PhantomData<(T, F)>);

    /// Union lense option
    unsafe impl<T, S> Lense<S> for Option<T>
        where T: Lense<S> + SizedLense,
              S: Ref
    {
        type Ret = option::Option<T::Ret>;

        fn lense(c: &mut Cursor<S>) -> Res<Self::Ret> {
            c.advance(T::size() as u64)
             .and_then(|ref mut c|
                if try!(bool::lense(c)).get() {
                    Ok(Some(try!(T::lense(c))))
                } else {
                    Ok(None)
                })
        }
    }

    unsafe impl<L, R> SizedLense for Result<L, R>
        where L: SizedLense,
              R: SizedLense
    {
        fn size() -> usize { *[L::size(), R::size()].iter().max().unwrap() }
    }

    /// Union lense result
    unsafe impl<L, R, S> Lense<S> for Result<L, R>
        where L: Lense<S> + SizedLense,
              R: Lense<S> + SizedLense,
              S: Ref
    {
        type Ret = result::Result<L::Ret, R::Ret>;

        fn lense(c: &mut Cursor<S>) -> Res<Self::Ret> {
            c.advance(Self::size() as u64)
             .and_then(|ref mut c|
                if try!(bool::lense(c)).get() {
                    Ok(Ok(try!(L::lense(c))))
                } else {
                    Ok(Err(try!(R::lense(c))))
                })
        }
    }
}

/// Tag managed boolean
unsafe impl<S> Lense<S> for bool where S: Ref {
    type Ret = BoolGuard<S>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        c.tag_bit()
    }
}

/// TODO: Handle endianness correctly
pub struct BoolGuard<S> {
    tag: *mut u8,
    mask: u8,
    marker: marker::PhantomData<S>
}

impl<S> BoolGuard<S> where S: Ref {
    fn new(tag: *mut u8, mask: u8) -> Self {
        BoolGuard {
            tag: tag,
            mask: mask,
            marker: marker::PhantomData,
        }
    }

    /// Get the current value of the boolean
    pub fn get(&self) -> bool {
        unsafe {
            *self.tag & self.mask == self.mask
        }
    }

    /// Set the boolean in the shared tag
    pub fn set(&mut self, b: bool) where S: Mut {
        unsafe {
            if b {
                *self.tag |= self.mask
            } else {
                *self.tag &= !self.mask
            }
        }
    }
}
