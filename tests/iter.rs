extern crate lense;

use lense::SeekablePool;

#[test]
fn prim_immutable_iter() {
    let pool = SeekablePool::<u16>::with_capacity(1);
    for guard in pool.iter() {
        assert_eq!(**guard, 0u16);
    }
}

#[test]
fn prim_mutable_iter() {
    let mut pool = SeekablePool::<u16>::with_capacity(1);
    for mut guard in pool.iter_mut() {
        **guard = 12345;
    }
}
