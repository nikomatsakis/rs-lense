use std::{marker, ops};

use {Lense, Cursor, Result, SizedLense, DstExt};

/// Vector for sized lenses
unsafe impl<'a, T: Lense<'a, S> + SizedLense, S> Lense<'a, S> for Vec<T>
    where S: ops::Deref<Target=[u8]>
{
    type Ret = Iter<T, S>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        let l = *try!(<u16>::lense(c)) as usize * T::size();
        c.advance(l as u64).map(Iter::new)
    }
}

impl<'a, T: Lense<'a, S> + SizedLense, S> DstExt<'a, S, T> for Vec<T>
    where S: ops::Deref<Target=[u8]> + ops::DerefMut
{
    type Ret = Iter<T, S>;

    fn with_length(c: &mut Cursor<S>, l: u16) -> Result<Self::Ret> {
        *try!(<u16>::lense(c)) = l;
        c.advance((l as usize * T::size()) as u64).map(Iter::new)
    }
}

/// Immutable iterator
struct Iter<T: SizedLense, S> {
    cursor: Cursor<S>,
    marker: marker::PhantomData<*mut T>,
}

impl<T: SizedLense, S> Iter<T, S> {
    fn new(c: Cursor<S>) -> Self {
        Iter {
            cursor: c,
            marker: marker::PhantomData,
        }
    }
}

impl<'a, S, T: Lense<'a, S> + SizedLense> Iterator for Iter<T, S> {
    type Item = T::Ret;

    fn next(&mut self) -> Option<Self::Item> {
        T::lense(&mut self.cursor).ok()
    }
}
