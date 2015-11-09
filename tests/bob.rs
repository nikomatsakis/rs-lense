#![cfg(feature = "automatic_padding")]
#[macro_use] extern crate lense;

use lense::Lense;

// Bad padding leading to 8 wasted bytes
mk_lense_ty!{struct BobRef ref
    _a:  u8,   // 1
               // 1 padding
    _bc: (u16, // 2
          u8), // 1
               // 1 padding
    _d:  u32,  // 4
    _e:  u64,  // 8
               // 6 padding
} // 1 + 1 + 2 + 1 + 1 + 4 + 8 + 6 = 24

#[test]
#[ignore]
fn size_bob_24_padded() {
    assert_eq!(BobRef::size(), 24);
}
