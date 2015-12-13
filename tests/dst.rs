extern crate lense;

use lense::{Aligned, Cursor, DstExt, Lense};

#[test]
fn write_then_read_slice() {
    let mut v = Aligned::new(8);

    { // writer
        let ref mut c = Cursor::new(&mut *v);

        let mut slice = <[u8]>::set_length(c, 4).unwrap();
        let mut tail = <u16>::lense(c).unwrap();
        for (n, mut a) in slice.iter_mut().enumerate() {
            *a = n as u8;
        }
        tail.set(0x0605);
    }

    // reader
    let ref mut c = Cursor::new(&*v);
    // Vector of length 4, containing [0, 1, 2, 3]
    let s = <[u8]>::lense(c);
    // Number 0x0605
    let t = <u16>::lense(c);

    // Assert data is read as written.
    assert_eq!(&*s.unwrap(), &[0, 1, 2, 3]);
    assert_eq!(t.unwrap().get(), 0x0605);

    // Everything is aligned; no waste!
    assert_eq!(c.waste(), 0);
    assert_eq!(c.remaining(), 0);
}

#[test]
fn write_then_read_vec() {
    let mut v = Aligned::new(8);

    { // writer
        let ref mut c = Cursor::new(&mut *v);

        let vec = <Vec<u8>>::set_length(c, 4).unwrap();
        let mut tail = <u16>::lense(c).unwrap();
        for (n, mut a) in vec.enumerate() {
            a.set(n as u8);
        }
        tail.set(0x0605);
    }

    // reader
    let ref mut c = Cursor::new(&*v);
    // Vector of length 4, containing [0, 1, 2, 3]
    let s = <Vec<u8>>::lense(c);
    // Number 0x0605
    let t = <u16>::lense(c);

    // Assert data is read as written.
    assert_eq!(&*s.unwrap()
                  .map(|x| x.get())
                  .collect::<Vec<_>>(),
               &[0, 1, 2, 3]);
    assert_eq!(t.unwrap().get(), 0x0605);

    // Everything is aligned; no waste!
    assert_eq!(c.waste(), 0);
    assert_eq!(c.remaining(), 0);
}
