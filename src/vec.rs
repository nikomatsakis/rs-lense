use std::{io, marker, ops, slice};
use std::io::Seek;

use {Lense, Cursor, Result, Error, SizedLense};

/// Vector for sized lenses
unsafe impl<'a, T: Lense<'a> + SizedLense> Lense<'a> for Vec<T> {
    type Ref = Iter<'a, T>;
    type Mut = IterMut<'a, T>;

    fn lense<S>(c: &mut Cursor<S>) -> Result<Self::Ref>
        where S: ops::Deref<Target=[u8]>
    {
        let l = *try!(<u16>::lense(c)) as usize * T::size() ;
        let p = c.as_ptr();
        match c.seek(io::SeekFrom::Current(l as i64)) {
            Ok(_) => Ok(unsafe { Iter::new(slice::from_raw_parts(p as *const u8, l)) }),
            Err(_) => Err(Error::OutOfBounds(l)),
        }
    }

    fn lense_mut<S>(c: &mut Cursor<S>) -> Result<Self::Mut>
        where S: ops::Deref<Target=[u8]> + ops::DerefMut
    {
        let l = *try!(<u16>::lense(c)) as usize * T::size() ;
        let p = c.as_ptr();
        match c.seek(io::SeekFrom::Current(l as i64)) {
            Ok(_) => Ok(unsafe { IterMut::new(slice::from_raw_parts_mut(p as *mut u8, l)) }),
            Err(_) => Err(Error::OutOfBounds(l)),
        }
    }
}

/// Immutable iterator
struct Iter<'a, T: SizedLense> {
    cursor: Cursor<&'a [u8]>,
    marker: marker::PhantomData<T>,
}

impl<'a, T: SizedLense> Iter<'a, T> {
    fn new(c: &'a [u8]) -> Self {
        Iter {
            cursor: Cursor::new(c),
            marker: marker::PhantomData,
        }
    }
}

impl<'a, T: Lense<'a> + SizedLense> Iterator for Iter<'a, T> {
    type Item = T::Ref;

    fn next(&mut self) -> Option<Self::Item> {
        T::lense(&mut self.cursor).ok()
    }
}

/// Mutable iterator
struct IterMut<'a, T: SizedLense> {
    cursor: Cursor<&'a mut [u8]>,
    marker: marker::PhantomData<T>,
}

impl<'a, T: SizedLense> IterMut<'a, T> {
    fn new(c: &'a mut [u8]) -> Self {
        IterMut {
            cursor: Cursor::new(c),
            marker: marker::PhantomData,
        }
    }
}

impl<'a, T: Lense<'a> + SizedLense> Iterator for IterMut<'a, T> {
    type Item = T::Mut;

    fn next(&mut self) -> Option<Self::Item> {
        T::lense_mut(&mut self.cursor).ok()
    }
}
