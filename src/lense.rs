use std::{mem, ops};

use {Cursor, Primitive, Result, SizedLense};

macro_rules! lense {
    (ref($cursor:ident) $lense:expr) => {
        fn lense<S>($cursor: &mut Cursor<S>) -> Result<Self::Ref>
            where S: ops::Deref<Target=[u8]>
        {
            $lense
        }
    };

    (mut($cursor:ident) $lense:expr) => {
        fn lense_mut<S>($cursor: &mut Cursor<S>) -> Result<Self::Mut>
            where S: ops::Deref<Target=[u8]> + ops::DerefMut
        {
            $lense
        }
    }
}

unsafe impl<P: Primitive> SizedLense for P {
    fn size() -> usize { mem::size_of::<P>() }
}

/// Read a lense-safe type out of the cursor!
pub unsafe trait Lense<'a> {
    type Ref;
    type Mut;

    fn lense<S>(&mut Cursor<S>) -> Result<Self::Ref>
        where S: ops::Deref<Target=[u8]>;
    fn lense_mut<S>(&mut Cursor<S>) -> Result<Self::Mut>
        where S: ops::Deref<Target=[u8]> + ops::DerefMut;
}

unsafe impl<'a, P: 'a + Primitive> Lense<'a> for P {
    type Ref = &'a P;
    type Mut = &'a mut P;

    lense!(ref(c) Ok(unsafe { &*try!(c.dice()) }) );
    lense!(mut(c) Ok(unsafe { &mut *try!(c.dice_mut()) }) );
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
        unsafe impl<'a, $($tail: Lense<'a>),*> Lense<'a> for ($($tail,)*) {
            type Ref = ($($tail::Ref,)*);
            type Mut = ($($tail::Mut,)*);

            lense!(ref(_c) Ok(($(try!($tail::lense(_c)),)*)) );
            lense!(mut(_c) Ok(($(try!($tail::lense_mut(_c)),)*)) );
        }
        impls!{ () $($tail)* }
    };

    ([]) => {};
    ([] ($n:expr) $(($m:expr))*) => {
        unsafe impl<L: SizedLense> SizedLense for [L; $n] {
            fn size() -> usize { L::size() * $n }
        }
        unsafe impl<'a, L: Lense<'a>> Lense<'a> for [L; $n] {
            type Ref = [L::Ref; $n];
            type Mut = [L::Mut; $n];

            lense!(ref(_c) Ok([$(impls!{ @void $m try!(L::lense(_c)) },)*]) );
            lense!(mut(_c) Ok([$(impls!{ @void $m try!(L::lense_mut(_c)) },)*]) );
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
