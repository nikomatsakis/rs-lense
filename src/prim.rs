use std::mem;

/// Cannot be implemented on types with pointers
pub unsafe trait Primitive {}

/// Unsafe because it does not account for padding?
pub unsafe trait SizedLense {
    fn size() -> usize;
    // Does not account for padding yet..
}

/// Unsafe trait marker for aligned asserted lenses. Do not manually implement! Must use
/// lense_aligned!(..)
pub unsafe trait AlignedLense: SizedLense {}

unsafe impl<P> SizedLense for P
    where P: Primitive
{
    fn size() -> usize { mem::size_of::<P>() }
}

unsafe impl<P> AlignedLense for P
    where P: Primitive { }

macro_rules! impls {
    ($($t:ident)*) => {$(
        unsafe impl Primitive for $t { }
    )*}
}

impls!{
    u8 u16 u32 u64
    i8 i16 i32 i64
}
