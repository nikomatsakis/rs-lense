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
    - [ ] Automatic padding occurs at runtime and skips this check

Lense safe types
----------------

A type is lense-safe if it is `Sized` and does not contain any pointers.
Consequently primitive and compositional types are lense-safe while `Vec` and
`HashMap` are not.

Traits
------

**Dice**: Chop the current slice into two segments, advance the slice and
return the lense.

**Slice**: Wrapper around `Dice` for primitive and compositive types.

Safety policy
-------------

The **only** unsafe code in `lense` is defined in the `Dice` trait in order to
cast the raw pointers, to the appropriate lense-safe types.

The **ONLY** unsafe code is defined in the `Dice` trait and for performance
reasons neither `Dice` nor `Slice` perform length checks on the buffer.
Instead, length checks occur in `Iter` and `IterMut` which inherently also
applies to the `LenseFile` struct.

Usage
-----

**This section needs to be re-written due to updates**

The following example is `examples/alice.rs` and can be ran with `cargo run --example alice`

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

Benchmarks
----------

```
Linux livecd 3.18.9-hardened #1 SMP x86_64 GNU/Linux
Intel(R) Atom(TM) CPU N450 @ 1.66GHz GenuineIntel
```
```
running 4 tests
test alice_writer     ... bench:          10 ns/iter (+/- 1) = 1500 MB/s
test alice_x3_reader  ... bench:         132 ns/iter (+/- 7) = 340 MB/s
test u64x32x32_reader ... bench:       8,677 ns/iter (+/- 182) = 944 MB/s
test u64x32x32_writer ... bench:       9,655 ns/iter (+/- 288) = 848 MB/s
```
