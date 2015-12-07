#![feature(test)]

#[macro_use]
extern crate lense;
extern crate test;

use test::{Bencher, black_box};
use lense::{Lense, Cursor, Aligned};

type U64x32x32 = [[u64; 32]; 32];

#[bench]
fn u64_8k_read(b: &mut Bencher) {
    let vec = Aligned::new(32 * 32);
    b.bytes = 32 * 32 * 8;
    b.iter(|| black_box(<U64x32x32>::lense(&mut Cursor::new(&*vec))) );
}
