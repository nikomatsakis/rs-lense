#![feature(slice_patterns)]

//! MIT 2015 DarkFox

#[macro_use]
mod prim;
mod file;
mod seekable;
mod aligned;

pub use seekable::{IterRef, IterMut, SeekablePool};
pub use file::LenseFile;

/// Core lense trait
pub trait Lense {
    fn size() -> usize;
    // alignment?
}

pub trait SliceRef<'a>: Lense {
    type Ref;
    fn slice<B: Dice<'a>>(&mut B) -> Self::Ref;
}

pub trait SliceMut<'a>: Lense {
    type Mut;
    fn slice_mut<B: DiceMut<'a>>(&mut B) -> Self::Mut;
}

pub trait Dice<'a>: Sized {
    fn dice<L: Lense>(&mut self) -> &'a L;
    fn size(&self) -> usize;
}

pub trait DiceMut<'a>: Dice<'a> {
    fn dice_mut<L: Lense>(&mut self) -> &'a mut L;
}

macro_rules! mk_dice {
    (mut $ty:ty, $x:expr, $split:ident) => (
        mk_dice!{$ty, $x, $split}
        impl<'a> DiceMut<'a> for $ty {
            #[inline]
            fn dice_mut<L: Lense>(&mut self) -> &'a mut L {
                let (head, tail) = ::std::mem::replace(self, $x).$split(L::size());
                *self = tail;
                unsafe { &mut *(head.as_ptr() as *mut L) }
            }
        }
    );
    ($ty:ty, $x:expr, $split:ident) => (
        impl<'a> Dice<'a> for $ty {
            #[inline]
            fn dice<L: Lense>(&mut self) -> &'a L {
                let (head, tail) = ::std::mem::replace(self, $x).$split(L::size());
                *self = tail;
                unsafe { &*(head.as_ptr() as *const L) }
            }

            #[inline]
            fn size(&self) -> usize {
                self.len()
            }
        }
    );
}

mk_dice!{    &'a     [u8], &[],     split_at}
mk_dice!{mut &'a mut [u8], &mut [], split_at_mut}
