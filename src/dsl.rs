/// DSL for constructing lense types
/// ================================
///
/// - Evolve-ty is an optional extension which would need to re-implement this macro
///
// lense_dsl!{
//     /// Documentation is good
//     pub struct Alice {
//         pub mut a: u8,
//         pub b: u16,
//         mut c: u32,
//         c: u64,
//     } // size 8 + 4 + 2 + 1 = 15; align = 8
//
//     /// Multiple items may be managed at once
//     enum Bob {
//         A(u8),
//         B(u16),
//     } // size: 1 + max(1, 2) = 3; align = 2
//
//     /// Network packet for some protocol
//     struct Packet {
//         header: [u8], // size 2+n, align 1
//         data: Carol,
// //        > Alice  // size 15; align 8
// //        > Bob    // size  3; align 2
//         tail: u64, // size  8, align 8
//     }
// }
//
// Packet sample 1
// ---------------
//
// header: 03
//         00 00 00
// // pad: 00 00 00
// // tag: 00
// alice:  00
//         00 00
//         00 00 00 00
//         00 00 00 00 00 00 00 00
// // pad: 00
// tail:   00 00 00 00 00 00 00 00
//
// // size: 32, align: 8
//
//
// Packet sample 2
// ---------------
//
// header: 03
//         00 00 00
// // tag: 01
// bob:    00 // tag
//         00 00 // u8 or u16
// tail:   00 00 00 00 00 00 00 00
//
// // size: 16, align: 8
#[macro_export]
macro_rules! lense_dsl {
    () => {}
}
