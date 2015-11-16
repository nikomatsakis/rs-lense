#![feature(test)]
#![allow(dead_code)]

#[macro_use]
extern crate lense;
extern crate test;

use test::{Bencher, black_box};
use lense::{Lense, SeekablePool, IsRef, IsMut};

mk_lense_struct!{pub struct Alice:
    a:  u8,        // 1
    bc: (u8, u16), // 3
    d:  u32,       // 4
    e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

type TupleAlice = (u8, (u8, u16), u32, u64);
type U64x32x32 = [[u64; 32]; 32];

macro_rules! bench {
    (@as_expr $expr:expr) => { $expr };
    (fn $ident:ident(mut $pool:ident) for ($ty:ty) * $cap:tt $expr:expr) => {
        #[bench]
        fn $ident(b: &mut Bencher) {
            let mut $pool = black_box(SeekablePool::<$ty>::with_capacity(bench!(@as_expr $cap)));
            b.bytes = <$ty as Lense>::size() as u64 * bench!(@as_expr $cap);
            b.iter(|| $expr);
        }
    };
    (fn $ident:ident($pool:ident) for ($ty:ty) * $cap:tt $expr:expr) => {
        #[bench]
        fn $ident(b: &mut Bencher) {
            let $pool = black_box(SeekablePool::<$ty>::with_capacity(bench!(@as_expr $cap)));
            b.bytes = <$ty as Lense>::size() as u64 * bench!(@as_expr $cap);
            b.iter(|| $expr);
        }
    };
    ($(fn $ident:ident $args:tt for $ty:tt * $cap:tt { $expr:expr })*) => {
        $( bench!{ fn $ident $args for $ty * $cap $expr } )*
    };
}

bench!{
    fn tuple_alice_x3_iter(pool) for (TupleAlice) * 3 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    fn tuple_alice_x3_iter_mut(mut pool) for (TupleAlice) * 3 {
        for mut guard in pool.iter_mut() {
            *guard.0 = 1;
            *guard.1 .0 = 2;
            *guard.1 .1 = 3;
            *guard.2 = 4;
            *guard.3 = 5;
        }
    }

    fn struct_alice_x3_iter(pool) for (Alice<IsRef>) * 3 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    fn struct_alice_x3_iter_mut(mut pool) for (Alice<IsMut>) * 3 {
        for mut guard in pool.iter_mut() {
            *guard.a = 1;
            *guard.bc.0 = 2;
            *guard.bc.1 = 3;
            *guard.d = 4;
            *guard.e = 5;
        }
    }

    fn u64_8k_iter(pool) for (U64x32x32) * 8 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    fn u64_8k_iter_mut(mut pool) for (U64x32x32) * 8 {
        for mut guard in pool.iter_mut() {
            for i in 0..32 {
                for j in 0..32 {
                    *guard[i][j] = 1u64;
                }
            }
        }
    }

    fn u64_64k_iter(pool) for (U64x32x32) * 16 {
        for guard in pool.iter() {
            black_box(guard);
        }
    }

    fn u64_64k_iter_mut(mut pool) for (U64x32x32) * 16 {
        for mut guard in pool.iter_mut() {
            for i in 0..32 {
                for j in 0..32 {
                    *guard[i][j] = 1u64;
                }
            }
        }
    }
}
