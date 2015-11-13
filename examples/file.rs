#[macro_use] extern crate lense;

use std::fs::File;
use std::io::{Seek, SeekFrom};
use lense::{Lense, LenseFile, IsRef};

mk_lense_struct!{pub struct Alice:
    /// Fields may be documented
    pub a:  u8,    // 1
    bc: (u8, u16), // 3
    d:  u32,       // 4
    e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

// This lense can also be represented as a simple tuple:
// type Alice = (u8, (u8, u16), u32, u64);

// ~lense.git $ hexdump -C lense-testing-file.dat
// 00000000  00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f
// *
// 00000050
//           a. b. c. .. d. .. .. ..  e. .. .. .. .. .. .. ..

fn main() {
    // Open a testing file containing the raw binary as displayed above with hexdump.
    let mut file = File::open("lense-testing-file.dat").unwrap();
    // Prepare a SeekablePool with the capcity to store 5 instances of Alice.
    let mut lf = LenseFile::<Alice<_>>::with_capacity(5);

    // Seek the file past the first 2 entries.
    let _current_position = file.seek(SeekFrom::Start(Alice::<IsRef>::size() as u64 * 2));

    // Read the remaining 3 entries directly into the pool and assert that we read exactly
    // Alice::size() * 3.
    assert_eq!(lf.read_file(&mut file).unwrap(), Alice::<IsRef>::size() * 3);

    // New scope to manually iterate over the pool and mutate relevent entries.
    {
        // Skip two entries, then increment the first value of the third by 5.
        let mut it = lf.iter_mut().skip(2);
        if let Some(mut guard) = it.next() {
            let Alice { ref mut a, .. } = *guard;
            **a += 5;
        }

        // Create the 4th entry manually.
        if let Some(mut guard) = it.next() {
            let Alice { ref mut a, bc: (ref mut b, ref mut c), ref mut d, ref mut e }
                = *guard;
            **a = 2;
            **b = 4;
            **c = 8;
            **d = 16;
            **e = 32;
        }
    }

    // Iterate over all entries currently stored in the pool
    for guard in lf.iter() {
        // The guard locks the current index because we currently own the access.

        // Deconstruct the AliceRef struct into the respective fields for quick access.
        let Alice { a, bc: (b, c), d, e } = *guard;

        // Dump all values directly to stdout.
        println!("a: {}, b: {}, c: {}, d: {}, e: {}",
                 *a, *b, *c, *d, *e);

        // The guard is dropped and the current index is unlocked.
    }

    // The file is dropped and the pool is destroyed.
}
