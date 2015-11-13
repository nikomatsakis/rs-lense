#![feature(slice_patterns)]

//! MIT 2015 DarkFox

#[macro_use]
mod prim;
mod mode;
mod file;
mod seekable;
mod aligned;
pub mod macro_tests;

pub use mode::{Mode, IsRef, IsMut};
pub use seekable::{IterRef, IterMut, SeekablePool};
pub use file::LenseFile;

/// Return an immutable lense and advance the pointer
pub trait DiceRef {
    fn dice<'a, L: Lense>(&mut self) -> &'a L;
}

/// Return a mutable lense and advance the pointer
pub trait DiceMut: DiceRef {
    fn dice_mut<'a, L: Lense>(&mut self) -> &'a mut L;
}

/// Lense primitives need lifetimes
pub trait RefMut<'a>: Lense {
    type Ref;
    type Mut;
}

/// A lense-safe type
pub trait Lense: Mode<IsRef> {
    type Ref;
    fn size() -> usize;
    fn lense<Buf: DiceRef>(buf: &mut Buf) -> <Self as Mode<IsRef>>::Return;
}

/// A mutable lense-safe type
pub trait LenseMut: Lense + Mode<IsMut> {
    type Mut;
    fn lense_mut<Buf: DiceMut>(buf: &mut Buf) -> <Self as Mode<IsMut>>::Return;
}

macro_rules! mk_dice {
    (mut $ty:ty, $x:expr, $split:ident) => {
        impl<'a> DiceMut for $ty {
            fn dice_mut<'b, L: Lense>(&mut self) -> &'b mut L {
                let (head, tail) = ::std::mem::replace(self, $x).$split(L::size());
                *self = tail;
                unsafe { &mut *(head.as_mut_ptr() as *mut L) }
            }
        }
        mk_dice!{ $ty, $x, $split }
    };
    ($ty:ty, $x:expr, $split:ident) => {
        impl<'a> DiceRef for $ty {
            #[inline]
            fn dice<'b, L: Lense>(&mut self) -> &'b L {
                let (head, tail) = ::std::mem::replace(self, $x).$split(L::size());
                *self = tail;
                unsafe { &*(head.as_ptr() as *const L) }
            }
        }
    };
}

mk_dice!{     &'a     [u8], &[],    split_at }
mk_dice!{ mut &'a mut [u8], &mut[], split_at_mut }
