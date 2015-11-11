#[macro_use] extern crate lense;

use std::fs::File;
use lense::{Lense, LenseFile};

mk_lense_ty!{pub struct AliceRef ref
    a:  u8,        // 1
    bc: (u8, u16), // 3
    d:  u32,       // 4
    e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

// ~lense.git $ hexdump -C lense-testing-file.dat
// 00000000  00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f
// *
// 00000050
//           a. b. c. .. d. .. .. ..  e. .. .. .. .. .. .. ..

fn main() {
    // Open a testing file containing the raw binary as displayed above with hexdump.
    let mut file = File::open("lense-testing-file.dat").unwrap();
    // Prepare a SeekablePool with the capcity to store 5 instances of AliceRef.
    let mut lf = LenseFile::<AliceRef>::with_capacity(5);

    // Read the contents of the file 'lense-testing-file.dat' into the pool and assert
    // that we read exactly AliceRef::size() * 5 which has filled the pool. This file
    // *could* contain more entries and they would be ignored by this stage.
    assert_eq!(lf.read_file(&mut file).unwrap(), AliceRef::size() * 5);

    for guard in lf.iter() {
        // The guard locks the current index because we currently own the access.

        // Deconstruct the AliceRef struct into the respective fields for quick access.
        let AliceRef { a, bc: (b, c), d, e } = *guard;

        // Dump all values directly to stdout.
        println!("a: {}, b: {}, c: {}, d: {}, e: {}",
                 *a, *b, *c, *d, *e);

        // The guard is dropped and the current index is unlocked.
    }

    // The file is dropped and the pool is destroyed.
}
