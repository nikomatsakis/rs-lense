#[macro_use]
extern crate lense;

use lense::{Aligned, Cursor, Lense, Ref, Result as LResult, SizedLense, Tag};

lense_dsl!(@DSL
    struct Alice {
        a: u8,
        b: (u8, u16),
        c: [u32],
        d: Vec<u64>,
    }

    #[sized]
    enum Bob {
        A(u8),
        B(u8, u16),
    }

    enum Carol {
        Alice(@Alice),
        Bob(@Bob),
    }
);

// Manually implement Lense for Bob until enum impls are supported
unsafe impl<S> Lense<S> for Bob<S>
    where S: Ref
{
    type Ret = Self;

    fn lense(c: &mut Cursor<S>) -> LResult<Self::Ret> {
        c.advance(Self::size() as u64)
         .and_then(|ref mut c|
            if !try!(<bool>::lense(c)).get() {
                Ok(Bob::A(u8::lense(c).unwrap()))
            } else {
                Ok(Bob::B((u8::lense(c).unwrap()), u16::lense(c).unwrap()))
            })
    }
}

// Manually implement Lense for Bob until enum impls are supported
unsafe impl<S> Lense<S> for Carol<S>
    where S: Ref
{
    type Ret = Self;

    fn lense(c: &mut Cursor<S>) -> LResult<Self::Ret> {
        if !try!(<bool>::lense(c)).get() {
            Ok(Carol::Alice(try!(<Alice<_>>::lense(c))))
        } else {
            Ok(Carol::Bob(try!(<Bob<_>>::lense(c))))
        }
    }
}

#[test]
fn union_alice() {
    let v = Aligned::new(10);
    let ref mut c = Cursor::new(&*v);

    Tag::<u8>::lense(c).unwrap();

    match Carol::lense(c).unwrap() {
        Carol::Alice(Alice { a, b: (b1, b2), c, d }) => {
            assert_eq!(a.get(), 0);
            assert_eq!(b1.get(), 0u8);
            assert_eq!(b2.get(), 0u16);
            assert_eq!(&*c, &[]);
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

    { // Set union tag to Bob!
        let ref mut c = Cursor::new(&mut *v);
        // set tag, return nothing
        Tag::<u8>::lense(c).unwrap();

        // boolean from the tag
        <bool>::lense(c).unwrap().set(true);
    }

    let ref mut c = Cursor::new(&mut *v);

    Tag::<u8>::lense(c).unwrap();

    match Carol::lense(c).unwrap() {
        Carol::Bob(Bob::A(a)) => assert_eq!(a.get(), 0),
        _ => unreachable!()
    }

    assert_eq!(c.waste(), 0);
    assert_eq!(c.remaining(), 0);
}
