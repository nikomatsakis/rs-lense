#![feature(test)]
#![allow(dead_code)]

#[macro_use]
extern crate lense;
extern crate test;

use test::{Bencher, black_box};
use lense::{Lense, SeekablePool};

mk_lense_ty!{pub struct AliceRef ref
    a:  u8,        // 1
    bc: (u8, u16), // 3
    d:  u32,       // 4
    e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

mk_lense_ty!{pub struct AliceMut mut
    a:  u8,        // 1
    bc: (u8, u16), // 3
    d:  u32,       // 4
    e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

type TupleAlice = (u8, (u8, u16), u32, u64);
type U64x32x32 = [[u64; 32]; 32];

macro_rules! bench {
    (@as_expr $expr:expr) => { $expr };
    (fn $ident:ident(mut $pool:ident) for $ty:ident * $cap:tt $expr:expr) => {
        #[bench]
        fn $ident(b: &mut Bencher) {
            let mut $pool = black_box(SeekablePool::<$ty>::with_capacity(bench!(@as_expr $cap)));
            b.bytes = <$ty as Lense>::size() as u64 * bench!(@as_expr $cap);
            b.iter(|| $expr);
        }
    };
    (fn $ident:ident($pool:ident) for $ty:ident * $cap:tt $expr:expr) => {
        #[bench]
        fn $ident(b: &mut Bencher) {
            let $pool = black_box(SeekablePool::<$ty>::with_capacity(bench!(@as_expr $cap)));
            b.bytes = <$ty as Lense>::size() as u64 * bench!(@as_expr $cap);
            b.iter(|| $expr);
        }
    };
    ($(fn $ident:ident $args:tt for $ty:ident * $cap:tt { $expr:expr })*) => {
        $( bench!{ fn $ident $args for $ty * $cap $expr } )*
    };
}

bench!{
    fn tuple_alice_x3_iter(pool) for TupleAlice * 3 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    // Hacky... o.o
    fn tuple_alice_x3_iter_mut(mut pool) for TupleAlice * 3 {
        for guard in pool.iter_mut() {
            let (ref mut a, (ref mut b, ref mut c), ref mut d, ref mut e) = *black_box(guard);
            **a = 1;
            **b = 1;
            **c = 1;
            **d = 1;
            **e = 1;
        }
    }

    fn struct_alice_x3_iter(pool) for AliceRef * 3 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    // Lifetimes issue?
//  fn struct_alice_x3_iter_mut(mut pool) for AliceMut * 3 {
//      for guard in pool.iter_mut() {
//          let AliceMut { a, bc: (b, c), d, e } = *black_box(guard);
//          *a = 1;
//          *b = 1;
//          *c = 1;
//          *d = 1;
//          *e = 1;
//      }
//  }

    fn u64_8k_iter(pool) for U64x32x32 * 8 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    fn u64_8k_iter_mut(mut pool) for U64x32x32 * 8 {
        for mut guard in pool.iter_mut() {
            for i in 0..32 {
                for j in 0..32 {
                    *guard[i][j] = 1u64;
                }
            }
        }
    }

    fn u64_64k_iter(pool) for U64x32x32 * 16 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    fn u64_64k_iter_mut(mut pool) for U64x32x32 * 16 {
        for mut guard in pool.iter_mut() {
            for i in 0..32 {
                for j in 0..32 {
                    *guard[i][j] = 1u64;
                }
            }
        }
    }
}
