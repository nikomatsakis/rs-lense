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
fn alignment_correction() {
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
