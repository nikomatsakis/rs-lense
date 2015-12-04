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


#[cfg(test)]
mod test{
    use std::{io, ops};
    use std::io::Seek;

    use {Aligned, Cursor, Lense, SizedLense, Result, Error};

    // Struct *maybe sized*
    struct Alice<'a> {
        a: <u8 as Lense<'a>>::Ref,
        b: <(u8, u16) as Lense<'a>>::Ref,
        c: <[u32] as Lense<'a>>::Ref, // unsized
        d: <Vec<u64> as Lense<'a>>::Ref, // unsized
    }

    unsafe impl<'a> Lense<'a> for Alice<'a> {
        type Ref = Alice<'a>;
        type Mut = ();

        fn lense<S>(c: &mut Cursor<S>) -> Result<Self::Ref>
            where S: ops::Deref<Target=[u8]>
        {
            Ok(Alice {
                a: try!(<u8>::lense(c)),
                b: try!(<(u8, u16)>::lense(c)),
                c: try!(<[u32]>::lense(c)),
                d: try!(<Vec<u64>>::lense(c)),
            })
        }

        fn lense_mut<S>(_: &mut Cursor<S>) -> Result<Self::Mut>
            where S: ops::Deref<Target=[u8]> + ops::DerefMut
        {
            unimplemented!()
        }
    }


    // Enum *sized*
    enum Bob<'a> {
        A(<u8 as Lense<'a>>::Ref),
        B(<u8 as Lense<'a>>::Ref, <u16 as Lense<'a>>::Ref),
    }

    unsafe impl<'a> SizedLense for Bob<'a> {
        fn size() -> usize {
            *[<u8 as SizedLense>::size(), <u8 as SizedLense>::size() + <u16 as SizedLense>::size()]
                .iter().max().unwrap()
        }
    }

    unsafe impl<'a> Lense<'a> for Bob<'a> {
        type Ref = Bob<'a>;
        type Mut = ();

        fn lense<S>(c: &mut Cursor<S>) -> Result<Self::Ref>
            where S: ops::Deref<Target=[u8]>
        {
            let ref mut e = c.clone();
            match c.seek(io::SeekFrom::Current(Bob::size() as i64)) {
                Ok(_) => match *try!(<u8>::lense(e)) {
                        0 => Ok(Bob::A(try!(<u8>::lense(e)))),
                        1 => Ok(Bob::B(try!(<u8>::lense(e)), try!(<u16>::lense(e)))),
                        n => Err(Error::InvalidTag(n)),
                    },
                Err(_) => Err(Error::OutOfBounds(Bob::size())),
            }
        }

        fn lense_mut<S>(_: &mut Cursor<S>) -> Result<Self::Mut>
            where S: ops::Deref<Target=[u8]> + ops::DerefMut
        {
            unimplemented!()
        }
    }


    // Union *unsized*
    enum Carol<'a> {
        Alice(Alice<'a>),
        Bob(Bob<'a>),
    }

    unsafe impl<'a> Lense<'a> for Carol<'a> {
        type Ref = Carol<'a>;
        type Mut = ();

        fn lense<S>(c: &mut Cursor<S>) -> Result<Self::Ref>
            where S: ops::Deref<Target=[u8]>
        {
            match *try!(<u8>::lense(c)) {
                0 => Ok(Carol::Alice(try!(<Alice>::lense(c)))),
                1 => Ok(Carol::Bob(try!(<Bob>::lense(c)))),
                n => Err(Error::InvalidTag(n)),
            }
        }

        fn lense_mut<S>(_: &mut Cursor<S>) -> Result<Self::Mut>
            where S: ops::Deref<Target=[u8]> + ops::DerefMut
        {
            unimplemented!()
        }
    }


    #[test]
    fn union_alice() {
        let v = Aligned::new(10);
        let ref mut c = Cursor::new(&*v);

        match <Carol>::lense(c).unwrap() {
            Carol::Alice(Alice { a, b, c, d }) => {
                assert_eq!(*a, 0);
                assert_eq!(b, (&0u8, &0u16));
                assert_eq!(c, &[]);
                assert_eq!(d.count(), 0);
            },
            _ => unreachable!()
        }

        if cfg!(feature = "no_automatic_padding") {
            assert_eq!(c.waste(), 0);
            assert_eq!(c.remaining(), 1);
        } else {
            assert_eq!(c.waste(), 1);
            assert_eq!(c.remaining(), 0);
        }
    }

    #[test]
    fn union_bob() {
        let mut v = Aligned::new(4);
        let ref mut c = Cursor::new(&mut *v);

        { // Set union tag to Bob!
            let ref mut e = c.clone();
            *<u8>::lense_mut(e).unwrap() = 1;
        }

        match <Carol>::lense(c).unwrap() {
            Carol::Bob(Bob::A(a)) => assert_eq!(*a, 0),
            _ => unreachable!()
        }

        assert_eq!(c.waste(), 0);
        assert_eq!(c.remaining(), 0);
    }
}
