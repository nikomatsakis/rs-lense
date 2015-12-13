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

#[macro_export]
macro_rules! lense_sized {
    ($ident:ty, $($ty:ty),*) => {
        unsafe impl<S> $crate::SizedLense for $ident
            where S: ::std::ops::Deref<Target=[u8]>
        {
            fn size() -> usize { 0usize $(+ <$ty>::size())* }
        }
    }
}

#[macro_export]
macro_rules! lense_aligned {
    ($ident:ident, $test:ident) => {
        unsafe impl<S> $crate::AlignedLense for $ident<S>
            where S: ::std::ops::Deref<Target=[u8]> {}

        #[test]
        fn $test() {
            use $crate::{Lense, SizedLense, Aligned, Cursor};

            let v = Aligned::new(<$ident<&[u8]>>::size() * 3);
            let ref mut c = Cursor::new(&*v);
            for _ in 0..3 {
                $ident::lense(c).unwrap();
            }
            assert_eq!(c.waste(), 0);
        }
    }
}
