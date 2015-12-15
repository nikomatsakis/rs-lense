use std::{marker, mem, ops, slice};

use {Cursor, DstExt, Lense, Mut, Primitive, Ref, Result};

unsafe impl<L, S> Lense<S> for [L] where L: Primitive, S: Ref {
    type Ret = Slice<L, S>;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        let l = try!(<u16>::lense(c)).get() as usize;
        try!(c.align::<L>());
        c.advance((l * mem::size_of::<L>()) as u64)
         .map(|c| Slice::new(c, l as usize) )
    }
}

impl<L, S> DstExt<S, L> for [L] where L: Primitive + Lense<S>, S: Ref {
    type Ret = Slice<L, S>;

    fn set_length(c: &mut Cursor<S>, l: u16) -> Result<Self::Ret> where S: Mut {
        try!(<u16>::lense(c)).set(l);
        Self::with_length(c, l)
    }

    fn with_length(c: &mut Cursor<S>, l: u16) -> Result<Self::Ret> {
        c.advance((l as usize * mem::size_of::<L>()) as u64)
         .map(|c| Slice::new(c, l as usize) )
    }
}

pub struct Slice<P, S> where P: Primitive, S: Ref {
    cursor: Cursor<S>,
    length: usize,
    marker: marker::PhantomData<*mut P>,
}

impl<P, S> Slice<P, S> where P: Primitive, S: Ref {
    fn new(c: Cursor<S>, l: usize) -> Self {
        Slice {
            cursor: c,
            length: l,
            marker: marker::PhantomData,
        }
    }
}

impl<P, S> ops::Deref for Slice<P, S> where P: Primitive, S: Ref {
    type Target = [P];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.cursor.as_ptr() as *const _, self.length)
        }
    }
}

impl<P, S> ops::DerefMut for Slice<P, S> where P: Primitive, S: Mut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.cursor.as_mut_ptr() as *mut _, self.length)
        }
    }
}
