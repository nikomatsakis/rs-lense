use std::{mem, ops};

use {Cursor, Primitive, Result, SizedLense, RefMut};

unsafe impl<P: Primitive> SizedLense for P {
    fn size() -> usize { mem::size_of::<P>() }
}

/// Read a lense-safe type out of the cursor!
pub unsafe trait Lense<'a, S> {
    type Ret;

    fn lense(&mut Cursor<S>) -> Result<Self::Ret>;
}

unsafe impl<'a, P: 'a + Primitive, S> Lense<'a, S> for P where S: ops::Deref<Target=[u8]> {
    type Ret = RefMut<P, S>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        c.dice()
    }
}

macro_rules! impls {
    (@void $void:tt $expr:expr) => { $expr };
    (array $($tt:tt)*) => { impls!{ [] $(($tt))* } };
    (tuple $($tt:tt)*) => { impls!{ () void $($tt)* } };

    (()) => {};
    (() $void:tt $($tail:ident)*) => {
        unsafe impl<$($tail: SizedLense),*> SizedLense for ($($tail,)*) {
            fn size() -> usize { 0usize $(+ $tail::size())* }
        }
        unsafe impl<'a, S, $($tail: Lense<'a, S>),*> Lense<'a, S> for ($($tail,)*) {
            type Ret = ($($tail::Ret,)*);

            fn lense(_c: &mut Cursor<S>) -> Result<Self::Ret> {
                Ok(($(try!($tail::lense(_c)),)*))
            }
        }
        impls!{ () $($tail)* }
    };

    ([]) => {};
    ([] ($n:expr) $(($m:expr))*) => {
        unsafe impl<L: SizedLense> SizedLense for [L; $n] {
            fn size() -> usize { L::size() * $n }
        }
        unsafe impl<'a, L: Lense<'a, S>, S> Lense<'a, S> for [L; $n] {
            type Ret = [L::Ret; $n];

            fn lense(_c: &mut Cursor<S>) -> Result<Self::Ret> {
                Ok([$(impls!{ @void $m try!(L::lense(_c)) },)*])
            }
        }
        impls!{ [] $(($m))* }
    };
}

impls!{array
    32 31 30 29 28 27 26 25
    24 23 22 21 20 19 18 17
    16 15 14 13 12 11 10  9
     8  7  6  5  4  3  2  1
     0
}

impls!{tuple
    A B C D E F
    G H I J K L
}
