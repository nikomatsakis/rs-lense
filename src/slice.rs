use std::{ops, slice, io};
use std::io::Seek;

use {Lense, Cursor, Result, Error, Primitive};

// Slices only work for primitives, LenseVec for anything else
unsafe impl<'a, L: 'a + Primitive> Lense<'a> for [L] {
    type Ref = &'a Self;
    type Mut = &'a mut Self;

    fn lense<S>(c: &mut Cursor<S>) -> Result<Self::Ref>
        where S: ops::Deref<Target=[u8]>
    {
        let l = *try!(<u16>::lense(c)) as usize;
        let p = c.as_ptr();
        match c.seek(io::SeekFrom::Current(l as i64)) {
            Ok(_) => Ok(unsafe { slice::from_raw_parts(p as *const L, l) }),
            Err(_) => Err(Error::OutOfBounds(l)),
        }
    }

    fn lense_mut<S>(c: &mut Cursor<S>) -> Result<Self::Mut>
        where S: ops::Deref<Target=[u8]> + ops::DerefMut
    {
        let l = *try!(<u16>::lense(c)) as usize;
        let p = c.as_mut_ptr();
        match c.seek(io::SeekFrom::Current(l as i64)) {
            Ok(_) => Ok(unsafe { slice::from_raw_parts_mut(p as *mut L, l) }),
            Err(_) => Err(Error::OutOfBounds(l)),
        }
    }
}
