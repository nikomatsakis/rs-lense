#![feature(slice_patterns)]

extern crate lense;

use lense::{Aligned, Cursor, Lense};

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

    { // prepare DST
        let ref mut c = Cursor::new(&mut *v);
        *<u16>::lense(c).unwrap() = 4;
    }

    { // writer
        let ref mut c = Cursor::new(&mut *v);

        let slice = <[u8]>::lense(c).unwrap();
        let mut tail = <u16>::lense(c).unwrap();
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

    assert_eq!(<(u8, u16)>::lense(c)
                    .map(|(a, b)| (*a, *b))
                    .unwrap(), (0, 0));
    // Padding: 1
    autopad_check!(c.waste(), 2, 0);

    assert_eq!(<[(u8, u32); 4]>::lense(c)
                    .map(|[(a, b), (c, d), (e, f), (g, h)]|
                        [(*a, *b), (*c, *d), (*e, *f), (*g, *h)])
                    .unwrap(),
               [(0, 0), (0, 0), (0, 0), (0, 0)]);
    // Padding: 4
    autopad_check!(c.waste(), 6, 0);

    assert_eq!(<(u8, u8, u8, u8, u32, u32, u32, u32)>::lense(c)
                    .map(|(a, b, c, d, e, f, g, h)|
                        [(*a, *e), (*b, *f), (*c, *g), (*d, *h)])
                    .unwrap(),
               [(0, 0), (0, 0), (0, 0), (0, 0)]);
    // Padding: 0
    autopad_check!(c.waste(), 6, 0);
}
