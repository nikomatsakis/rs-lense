Rust lense macro
================

[![](https://img.shields.io/crates/v/lense.svg)](https://crates.io/crates/lense)
[![](https://img.shields.io/crates/d/lense.svg)](https://crates.io/crates/lense)

What is lense?
--------------

A lense allows you to peek into bytestreams and treat segments as if they were
fixed-width structs. Additionally the lense also allows the consumer to mutate
data easily, and safely using Rust's type system.

Lense does this by being a mutable by-ref reader that borrows a mutable
reference into a [u8] stream. Due to how little the lenses actually need to do,
there is no performance hit or slow step to serialising and deserialising data.

## Features

- No allocations (pre-allocated pool or `from_slice`)
- No copies
- No reference counters
- Minimal unsafety

## Optional features

- Aligned iterators
- Pre-allocated pools
- File backed ACID state database

## Possible use cases

- High performance stateless networking
- Streamed file format for storing big data (such as graph data)
- Maintaining the state of a program

## Warnings

- Endianness isn't touched in the buffer, you must handle this if you're doing
  networking or otherwise sharing across platforms.

Room for improvement
--------------------

- [ ] Variable length types (**must be known at writer time!**)
  - [ ] Union types
  - [ ] Vectors
    - [ ] Allocating should reserve a partition of the pool
    - [ ] Custom `Reader` and `Writer` to correctly handle I/O
  - [ ] HashMaps `Vec<(Key, Value>)::collect()`
- [ ] Automate padding to correct alignments
  - [x] Alignment type
  - [x] Aligned iterators
  - [ ] Calculate padding waste
  - [ ] Lint to complain when ordering is suboptimal
- Safety checks
  - [x] Iterators perform length checks before slicing the buffer
    - [ ] Automatic padding occurs at runtime and **doesn't perform this
      extended check**

Lense safe types
----------------

A type is lense-safe if it is `Sized` and does not contain any pointers.
Consequently primitive and compositional types are lense-safe while `Vec` and
`HashMap` are not.

Usage
-----

The following example is `examples/file.rs` and can be ran with `cargo run --example file`

```rust
#[macro_use] extern crate lense;

use std::fs::File;
use std::io::{Seek, SeekFrom};
use lense::{Lense, LenseFile, IsRef, IsMut};

mk_lense_struct!{
    /// structs and enums may be documented
    pub struct Alice:
        /// so can public fields
        pub a:  u8,    // 1
        bc: (u8, u16), // 3
        d:  u32,       // 4
        e:  u64,       // 8
} // 1 + 3 + 4 + 8 = 16

impl ::std::fmt::Debug for Alice<IsRef> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "a: {:?}, b: {:?}, c: {:?}, d: {:?}, e: {:?}",
            self.a, self.bc.0, self.bc.1, self.d, self.e)
    }
}

impl Alice<IsMut> {
    fn new(&mut self, a: u8, (b, c): (u8, u16), d: u32, e: u64) {
        *self.a = a;
        *self.bc.0 = b;
        *self.bc.1 = c;
        *self.d = d;
        *self.e = e;
    }
}

//// This lense can also be represented as a simple tuple.
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
            *guard.a += 5;
        }

        // Create the 4th entry manually.
        if let Some(mut guard) = it.next() {
            guard.new(2, (4, 8), 16, 32);
        }
    }

    // Iterate over all entries currently stored in the pool
    for guard in lf.iter() {
        // The guard locks the current index because we currently own the access.

        // Dump all values directly to stdout.
        println!("{:?}", *guard);

        // The guard is dropped and the current index is unlocked.
    }

    // The file is dropped and the pool is destroyed.
}
```

Benchmarks
----------

```
Linux gentoo #3 SMP x86_64 Intel(R)
Core(TM) i5-4250U CPU @ 1.30GHz GenuineIntel
```

```
running 8 tests
test struct_alice_x3_iter     ... bench:          19 ns/iter (+/- 0) = 2526 MB/s
test struct_alice_x3_iter_mut ... bench:          25 ns/iter (+/- 0) = 1920 MB/s
test tuple_alice_x3_iter      ... bench:          19 ns/iter (+/- 0) = 2526 MB/s
test tuple_alice_x3_iter_mut  ... bench:          25 ns/iter (+/- 0) = 1920 MB/s
test u64_64k_iter             ... bench:      22,737 ns/iter (+/- 195) = 5764 MB/s
test u64_64k_iter_mut         ... bench:      25,806 ns/iter (+/- 229) = 5079 MB/s
test u64_8k_iter              ... bench:      11,427 ns/iter (+/- 150) = 5735 MB/s
test u64_8k_iter_mut          ... bench:      12,966 ns/iter (+/- 47) = 5054 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 8 measured
```
