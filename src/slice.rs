use std::slice;

use {Lense, Cursor, Result, Primitive, SizedLense};

unsafe impl<'a, L: 'a + SizedLense + Primitive> Lense<'a, &'a [u8]> for [L] {
    type Ret = &'a [L];

    fn lense(c: &mut Cursor<&[u8]>) -> Result<Self::Ret> {
        let l = *try!(<u16>::lense(c)) as usize;
        try!(c.align::<L>());
        c.advance((l * L::size()) as u64)
         .map(|c| unsafe { slice::from_raw_parts(c.as_ptr() as *const _, l) })
    }
}

unsafe impl<'a, L: 'a + SizedLense + Primitive> Lense<'a, &'a mut [u8]> for [L] {
    type Ret = &'a mut [L];

    fn lense(c: &mut Cursor<&mut [u8]>) -> Result<Self::Ret> {
        let l = *try!(<u16>::lense(c)) as usize;
        try!(c.align::<L>());
        c.advance((l * L::size()) as u64)
         .map(|mut c| unsafe { slice::from_raw_parts_mut(c.as_mut_ptr() as *mut _, l) })
    }
}
