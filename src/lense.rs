use {AlignedLense, Cursor, Endian, Ref, RefMut, Result, SizedLense};

/// Compose lense-safe primitives
pub unsafe trait Lense<S> {
    type Ret;

    fn lense(&mut Cursor<S>) -> Result<Self::Ret>;
}

unsafe impl<P, S> Lense<S> for P where P: Endian, S: Ref {
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
        unsafe impl<$($tail: AlignedLense),*> AlignedLense for ($($tail,)*) {}
        unsafe impl<$($tail: SizedLense),*> SizedLense for ($($tail,)*) {
            fn size() -> usize { 0usize $(+ $tail::size())* }
        }
        unsafe impl<S, $($tail: Lense<S>),*> Lense<S> for ($($tail,)*) {
            type Ret = ($($tail::Ret,)*);

            fn lense(_c: &mut Cursor<S>) -> Result<Self::Ret> {
                Ok(($(try!($tail::lense(_c)),)*))
            }
        }
        impls!{ () $($tail)* }
    };

    ([]) => {};
    ([] ($n:expr) $(($m:expr))*) => {
        unsafe impl<L: AlignedLense> AlignedLense for [L; $n] {}
        unsafe impl<L: SizedLense> SizedLense for [L; $n] {
            fn size() -> usize { L::size() * $n }
        }
        unsafe impl<L: Lense<S>, S> Lense<S> for [L; $n] {
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
