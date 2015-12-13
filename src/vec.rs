use std::{marker, ops};

use {AlignedLense, Cursor, DstExt, Lense, Result, SizedLense};

/// Vector for sized lenses
unsafe impl<T, S> Lense<S> for Vec<T>
    where T: Lense<S> + AlignedLense,
          S: ops::Deref<Target=[u8]>
{
    type Ret = Iter<T, S>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        let l = try!(<u16>::lense(c)).get() as usize * T::size();
        c.advance(l as u64).map(Iter::new)
    }
}

impl<T, S> DstExt<S, T> for Vec<T>
    where T: Lense<S> + AlignedLense,
          S: ops::Deref<Target=[u8]>
{
    type Ret = Iter<T, S>;

    fn set_length(c: &mut Cursor<S>, l: u16) -> Result<Self::Ret>
        where S: ops::DerefMut
    {
        try!(<u16>::lense(c)).set(l);
        Self::with_length(c, l)
    }

    fn with_length(c: &mut Cursor<S>, l: u16) -> Result<Self::Ret> {
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

impl<S, T> Iterator for Iter<T, S>
    where T: Lense<S> + SizedLense
{
    type Item = T::Ret;

    fn next(&mut self) -> Option<Self::Item> {
        T::lense(&mut self.cursor).ok()
    }
}
