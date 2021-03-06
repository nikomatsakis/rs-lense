#[macro_use] extern crate lense;

use lense::{Lense, SeekablePool, IsRef};

mk_lense_struct!{pub struct Alice:
    a:  u8,        // 1
    bc: (u8, u16), // 3
    d:  u32,       // 4
    e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

type TupleAlice = (u8, (u8, u16), u32, u64);

#[test]
fn tuple_alice_iter() {
    let pool = SeekablePool::<TupleAlice>::with_capacity(1);
    for guard in pool.iter() {
        let (a, (b, c), d, e) = *guard;
        assert_eq!(*a, 0u8);
        assert_eq!(*b, 0u8);
        assert_eq!(*c, 0u16);
        assert_eq!(*d, 0u32);
        assert_eq!(*e, 0u64);
    }
}

#[test]
fn alice_iter() {
    let pool = SeekablePool::<Alice<IsRef>>::with_capacity(4);
    for guard in pool.iter() {
        let Alice { a, bc: (b, c), d, e } = *guard;
        assert_eq!(*a, 0u8);
        assert_eq!(*b, 0u8);
        assert_eq!(*c, 0u16);
        assert_eq!(*d, 0u32);
        assert_eq!(*e, 0u64);
    }
}

#[test]
fn size_alice_16() {
    assert_eq!(Alice::<::lense::IsRef>::size(), 16);
}
