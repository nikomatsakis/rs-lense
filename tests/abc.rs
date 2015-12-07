
extern crate lense;

use std::ops::Deref;
use lense::{Aligned, Cursor, Lense, Result, Error, SizedLense};

struct Alice<'a, S>
    where S: Deref<Target=[u8]>
{
    a: <u8 as Lense<'a, S>>::Ret,
    b: <(u8, u16) as Lense<'a, S>>::Ret,
    c: <[u32] as Lense<'a, S>>::Ret,
    d: <Vec<u64> as Lense<'a, S>>::Ret,
}

enum Bob<'a, S>
    where S: Deref<Target=[u8]>
{
    A(<u8 as Lense<'a, S>>::Ret),
    B(<u8 as Lense<'a, S>>::Ret, <u16 as Lense<'a, S>>::Ret),
}

enum Carol<'a, S>
    where S: Deref<Target=[u8]>
{
    Alice(<Alice<'a, S> as Lense<'a, S>>::Ret),
    Bob(<Bob<'a, S> as Lense<'a, S>>::Ret),
}

unsafe impl<'a, S> Lense<'a, S> for Alice<'a, S>
    where S: Deref<Target=[u8]>
{
    type Ret = Self;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        Ok(Alice {
            a: try!(<u8>::lense(c)),
            b: try!(<(u8, u16)>::lense(c)),
            c: try!(<[u32]>::lense(c)),
            d: try!(<Vec<u64>>::lense(c)),
        })
    }
}

unsafe impl<'a, S> SizedLense for Bob<'a, S>
    where S: Deref<Target=[u8]>
{
    fn size() -> usize {
        *[<u8 as SizedLense>::size(), <u8 as SizedLense>::size() + <u16 as SizedLense>::size()]
            .iter().max().unwrap()
    }
}

unsafe impl<'a, S> Lense<'a, S> for Bob<'a, S>
    where S: Deref<Target=[u8]>
{
    type Ret = Self;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        c.advance(Self::size() as u64)
         .and_then(|ref mut c|
            match *<u8>::lense(c).unwrap() {
                0 => Ok(Bob::A(<u8>::lense(c).unwrap())),
                1 => Ok(Bob::B((<u8>::lense(c).unwrap()), <u16>::lense(c).unwrap())),
                n => Err(Error::InvalidTag(n)),
            })
    }
}


unsafe impl<'a, S> Lense<'a, S> for Carol<'a, S>
    where S: Deref<Target=[u8]>
{
    type Ret = Self;

    fn lense(c: &mut Cursor<S>) -> Result<Self::Ret> {
        match *try!(<u8>::lense(c)) {
            0 => Ok(Carol::Alice(try!(<Alice<_>>::lense(c)))),
            1 => Ok(Carol::Bob(try!(<Bob<_>>::lense(c)))),
            n => Err(Error::InvalidTag(n)),
        }
    }
}

#[test]
fn union_alice() {
    let v = Aligned::new(10);
    let ref mut c = Cursor::new(&*v);

    match <Carol<_>>::lense(c).unwrap() {
        Carol::Alice(Alice { a, b: (b1, b2), c, d }) => {
            assert_eq!(*a, 0);
            assert_eq!(*b1, 0u8);
            assert_eq!(*b2, 0u16);
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
        *<u8>::lense(c).unwrap() = 1;
    }

    let ref mut c = Cursor::new(&mut *v);

    match <Carol<_>>::lense(c).unwrap() {
        Carol::Bob(Bob::A(a)) => assert_eq!(*a, 0),
        _ => unreachable!()
    }

    assert_eq!(c.waste(), 0);
    assert_eq!(c.remaining(), 0);
}
