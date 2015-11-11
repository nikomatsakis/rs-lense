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

Traits
------

**Dice**: Chop the current slice into two segments, advance the slice and
return the lense.

**Slice**: Wrapper around `Dice` for primitive and composed types.

Usage
-----

The following example is `examples/file.rs` and can be ran with `cargo run --example file`

```rust
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
```

<!--

**Old example, needs updating**

```rust
#[macro_use] extern crate lense;
use lense::*;

// Public struct Alice
lense_struct!{pub Alice:
    a:  u8,
    b: (u8, u8),
    c: [u8; 4],
    d: u64,
}

// Private struct Bob
lense_struct!{Bob:
    // Note the <'a> is inherited from struct Alice<'a> in which we don't see. This allows us to
    // work on our own struct types directly
    a: Alice<'a>,
}

fn main() {
    // Buffer containing 3x Alice
    let mut alice = vec![0x00, // a[0].a
                         0x01, 0x02, // a[0].b
                         0x03, 0x04, 0x05, 0x06, // a[0].c
                         0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, // a[0].d
                         0x00, // ...
                         0x01, 0x02,
                         0x03, 0x04, 0x05, 0x06,
                         0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                         0x00,
                         0x01, 0x02,
                         0x03, 0x04, 0x05, 0x06,
                         0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                         ];

    // New vector of Alice::size() ready to be used.
    let mut alice_writer = vec![0u8; Alice::size()];
    { // Populate our new vector using a lense
        let (mut alice_writer_lense, rest) = Alice::new(&mut alice_writer);
        assert!(rest.len() == 0);
        *alice_writer_lense.a = 0;
        *alice_writer_lense.b.0 = 0x01;
        *alice_writer_lense.b.1 = 0x02;
        *alice_writer_lense.c[0] = 0x03;
        *alice_writer_lense.c[1] = 0x04;
        *alice_writer_lense.c[2] = 0x05;
        *alice_writer_lense.c[3] = 0x06;
        *alice_writer_lense.d = 1012478732780767239;
    }

    // Check that our manually populated Alice is identical to the first Alice in the vector 'a'
    assert!(&*alice_writer == &alice[0..Alice::size()]);

    { // Read each Alice from 'a'
        let mut remaining = &mut *alice;
        while let Ok(Some(mut a)) = Alice::from_buf(&mut remaining) {
            *a.a += 1;
            println!("a: {}; b: {:?}; c: {:?}; d: {}", *a.a, a.b, a.c, *a.d);
        }
        // If there is any excess, it is still accessible through the 'remaining' variable.
        // Alternatively this can be used as a starting point in a pool that owns some
        // preallocated-large buffer.
    }

    println!("Mutated result: {:?}", &*alice);
}

```
Output altered for viewing
```
a: 1; b: (1, 2); c: [3, 4, 5, 6]; d: 1012478732780767239
a: 1; b: (1, 2); c: [3, 4, 5, 6]; d: 1012478732780767239
a: 1; b: (1, 2); c: [3, 4, 5, 6]; d: 1012478732780767239
Mutated result:
  [1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
   1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
   1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
```

-->

Benchmarks
----------

```
Linux gentoo #3 SMP x86_64 Intel(R)
Core(TM) i5-4250U CPU @ 1.30GHz GenuineIntel
```

```
running 7 tests
test struct_alice_x3_iter    ... bench:          19 ns/iter (+/- 1) = 2526 MB/s
test tuple_alice_x3_iter     ... bench:          19 ns/iter (+/- 0) = 2526 MB/s
test tuple_alice_x3_iter_mut ... bench:          25 ns/iter (+/- 0) = 1920 MB/s
test u64_64k_iter            ... bench:      23,064 ns/iter (+/- 398) = 5682 MB/s
test u64_64k_iter_mut        ... bench:      26,075 ns/iter (+/- 91) = 5026 MB/s
test u64_8k_iter             ... bench:      11,596 ns/iter (+/- 117) = 5651 MB/s
test u64_8k_iter_mut         ... bench:      13,126 ns/iter (+/- 58) = 4992 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 7 measured
```
