
use std::{io, marker, mem, ops, slice};

use {Primitive, Result, Error};

/// Cursor state for seeking into [u8] slices
pub struct Cursor<S> { // todo bind to reference S
    start: *mut u8,
    cur:   *mut u8,
    end:   *mut u8,
    waste: usize,
    marker: marker::PhantomData<*mut S>,
}

impl<S> Clone for Cursor<S> {
    fn clone(&self) -> Self {
        Cursor {
            start: self.start,
            cur: self.cur,
            end: self.end,
            waste: 0,
            marker: marker::PhantomData,
        }
    }
}

impl<S> io::Seek for Cursor<S> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let n = unsafe {
            match pos {
                io::SeekFrom::Start(n) => self.start.offset(n as isize),
                io::SeekFrom::End(n) => self.end.offset(-(n as isize)),
                io::SeekFrom::Current(n) => self.cur.offset(n as isize),
            }
        };

        if self.start <= n && n <= self.end {
            self.cur = n;
            Ok(self.cur as u64 - self.start as u64)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput,
                               "out of bounds!"))
        }
    }
}

impl<S> ops::Deref for Cursor<S> where S: ops::Deref<Target=[u8]> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.cur, self.end as usize - self.cur as usize)
        }
    }
}

impl<S> ops::DerefMut for Cursor<S> where S: ops::Deref<Target=[u8]> + ops::DerefMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.cur, self.end as usize - self.cur as usize)
        }
    }
}

impl<S> Cursor<S> where S: ops::Deref<Target=[u8]> {
    /// Start maintaining a cursor over some type that derefrences to [u8]
    pub fn new(s: S) -> Self {
        let p = s.as_ptr() as *mut u8;
        Cursor {
            start: p,
            cur: p,
            end: unsafe { p.offset(s.len() as isize) },
            waste: 0,
            marker: marker::PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.cur
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 where S: ops::DerefMut {
        self.cur
    }

    pub fn waste(&self) -> usize {
        self.waste
    }

    pub fn remaining(&self) -> usize {
        self.end as usize - self.cur as usize
    }

    #[cfg(feature = "no_automatic_padding")]
    fn align<T>(&mut self) -> Result<()> { Ok(()) }

    #[cfg(not(feature = "no_automatic_padding"))]
    fn align<T>(&mut self) -> Result<()> {
        let pad = self.cur as usize % mem::align_of::<T>();
        if pad > 0 {
            let n = unsafe { self.cur.offset((mem::align_of::<T>() - pad) as isize) };
            if n < self.end {
                self.waste += pad;
                self.cur = n;
                Ok(())
            } else {
                Err(Error::ExpectedBytes(pad))
            }
        } else {
            Ok(())
        }
    }

    /// Unsafe because returns raw pointer
    pub unsafe fn dice<T: Primitive>(&mut self) -> Result<*const T> {
        try!(self.align::<T>());
        let c = self.cur;
        let n = self.cur.offset(mem::size_of::<T>() as isize);
        if n <= self.end {
            self.cur = n;
            Ok(c as *const T)
        } else {
            Err(Error::ExpectedBytes(mem::size_of::<T>()))
        }
    }

    /// Unsafe because returns raw pointer
    pub unsafe fn dice_mut<T: Primitive>(&mut self) -> Result<*mut T>
        where S: ops::DerefMut
    {
        try!(self.align::<T>());
        let c = self.cur;
        let n = c.offset(mem::size_of::<T>() as isize);
        if n <= self.end {
            self.cur = n;
            Ok(c as *mut T)
        } else {
            Err(Error::ExpectedBytes(mem::size_of::<T>()))
        }
    }
}

#[cfg(test)]
mod test {
    use {Aligned, Cursor, Lense};

    macro_rules! autopad_check {
        ($expr:expr, $l:expr, $r:expr) => {
            assert_eq!($expr,
                if cfg!(feature = "no_automatic_padding") {
                    $r
                } else {
                    $l
                })
        }
    }

    #[test]
    fn write_then_read() {
        let mut v = Aligned::new(8);

        { // writer
            let ref mut c = Cursor::new(&mut *v);

            { // Sub-cursor to prepare the slice
                let ref mut c = Cursor::new(&mut **c);
                *<u16>::lense_mut(c).unwrap() = 4;
            }

            let slice = <[u8]>::lense_mut(c).unwrap();
            let tail = <u16>::lense_mut(c).unwrap();
            slice[0] = 1;
            slice[1] = 2;
            slice[2] = 3;
            slice[3] = 4;
            *tail = 0x0605;
        }

        // reader
        let ref mut c = Cursor::new(&*v);
        // Slice of length 4, containing [1, 2, 3, 4]
        let s = <[u8]>::lense(c);
        // Number 0x0605
        let t = <u16>::lense(c);

        // Assert data is read as written.
        assert_eq!(s.unwrap(), &[1, 2, 3, 4]);
        assert_eq!(*t.unwrap(), 0x0605);

        // Everything is aligned; no waste!
        assert_eq!(c.waste(), 0)
    }

    #[test]
    fn bad_alignment() {
        let mut v = Aligned::new(64);
        let ref mut c = Cursor::new(&mut *v);

        assert_eq!(*<u8>::lense(c).unwrap(), 0);
        // Padding: 0
        autopad_check!(c.waste(), 0, 0);

        assert_eq!(*<u16>::lense(c).unwrap(), 0);
        // Padding: 1
        autopad_check!(c.waste(), 1, 0);

        assert_eq!(<(u8, u16)>::lense(c).unwrap(), (&0, &0));
        // Padding: 1
        autopad_check!(c.waste(), 2, 0);

        assert_eq!(<[(u8, u32); 4]>::lense(c).unwrap(),
                   [(&0u8, &0u32), (&0u8, &0u32),
                    (&0u8, &0u32), (&0u8, &0u32),
                   ]);
        // Padding: 4
        autopad_check!(c.waste(), 6, 0);

        assert_eq!(<(u8, u8, u8, u8, u32, u32, u32, u32)>::lense(c)
                        .map(|(a, b, c, d, e, f, g, h)|
                            [(a, e), (b, f), (c, g), (d, h)])
                        .unwrap(),
                   [(&0u8, &0u32), (&0u8, &0u32),
                    (&0u8, &0u32), (&0u8, &0u32),
                   ]);
        // Padding: 0
        autopad_check!(c.waste(), 6, 0);
    }
}
