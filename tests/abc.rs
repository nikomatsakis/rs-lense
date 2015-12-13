
extern crate lense;

use std::ops::Deref;
use lense::{Aligned, Cursor, Lense, Result as LResult, SizedLense, Tag};

struct Alice<S>
    where S: Deref<Target=[u8]>
{
    a: <u8 as Lense<S>>::Ret,
    b: <(u8, u16) as Lense<S>>::Ret,
    c: <[u32] as Lense<S>>::Ret,
    d: <Vec<u64> as Lense<S>>::Ret,
}

enum Bob<S>
    where S: Deref<Target=[u8]>
{
    A(<u8 as Lense<S>>::Ret),
    B(<u8 as Lense<S>>::Ret, <u16 as Lense<S>>::Ret),
}

enum Carol<S>
    where S: Deref<Target=[u8]>
{
    Alice(<Alice<S> as Lense<S>>::Ret),
    Bob(<Bob<S> as Lense<S>>::Ret),
}

unsafe impl<S> Lense<S> for Alice<S>
    where S: Deref<Target=[u8]>
{
    type Ret = Self;

    fn lense(c: &mut Cursor<S>) -> LResult<Self::Ret> {
        Ok(Alice {
            a: try!(<u8>::lense(c)),
            b: try!(<(u8, u16)>::lense(c)),
            c: try!(<[u32]>::lense(c)),
            d: try!(<Vec<u64>>::lense(c)),
        })
    }
}

unsafe impl<S> SizedLense for Bob<S>
    where S: Deref<Target=[u8]>
{
    fn size() -> usize {
        *[u8::size(), u8::size() + u16::size()]
            .iter().max().unwrap()
    }
}

unsafe impl<S> Lense<S> for Bob<S>
    where S: Deref<Target=[u8]>
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


unsafe impl<S> Lense<S> for Carol<S>
    where S: Deref<Target=[u8]>
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
