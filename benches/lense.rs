#![feature(test)]
extern crate test;
#[macro_use] extern crate lense;

use test::Bencher;

use lense::*;

make_lense!{PUB, Alice, AliceWriter,
    a:  u8,
    b: u16,
    c: [(u16); 2],
    d: u64,
} unsafe_writer!{Alice, AliceWriter}

#[bench]
fn alice_x3_reader(b: &mut Bencher) {
    let mut a = vec![0x00, // a[0].a
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
    b.bytes = 45;
    b.iter(|| {
        // Format: [(size, chunk); n] where n is number of chunks
        let mut pos = 0;
        loop {
            let (_, mut a) = a.split_at_mut(pos);
            match Alice::from_buf(&mut a) {
                Ok((mut a, size)) => { *a.a += pos as u8; pos += size },
                Err(LenseError::NothingToParse) => break, // no more to process :)
                Err(LenseError::UnexpectedSize) => break, // Invalid chunk
                Err(LenseError::Incomplete) => break, // Incomplete
            };
        }
        pos
    })
}

#[bench]
fn alice_writer_init(b: &mut Bencher) {
    b.bytes = 15;
    b.iter(move || {
        let mut a_ = Alice::writer();
        {
            let mut a_l = a_.borrow_lense();
            *a_l.a = 0;
            *a_l.b = 513;
            *a_l.c[0] = 123;
            *a_l.c[1] = 321;
            *a_l.d = 1012478732780767239;
        }
        a_
    })
}

#[bench]
fn alice_writer_uninit(b: &mut Bencher) {
    b.bytes = 15;
    b.iter(move || {
        let mut a_ = unsafe { Alice::writer_uninit() };
        {
            let mut a_l = a_.borrow_lense();
            *a_l.a = 0;
            *a_l.b = 513;
            *a_l.c[0] = 123;
            *a_l.c[1] = 321;
            *a_l.d = 1012478732780767239;
        }
        a_
    })
}

make_lense!{PUB, U64x2, U64x2Writer,
    x0: u64,
    x1: u64,
} unsafe_writer!{U64x2, U64x2Writer}

#[bench]
fn u64x2_reader(b: &mut Bencher) {
    b.bytes = 16;
    let mut a = vec![0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     ];
    b.iter(|| {
        let mut pos = 0;
        loop {
            let (_, mut a) = a.split_at_mut(pos);
            match U64x2::from_buf(&mut a) {
                Ok((mut a, size)) => { *a.x0 += pos as u64; pos += size },
                Err(LenseError::NothingToParse) => break, // no more to process :)
                Err(LenseError::UnexpectedSize) => break, // Invalid chunk
                Err(LenseError::Incomplete) => break, // Incomplete
            };
        }
        pos
    })
}

#[bench]
fn u64x2_writer_init(b: &mut Bencher) {
    b.bytes = 16;
    b.iter(move || {
        let mut a_ = U64x2::writer();
        {
            let mut a_l = a_.borrow_lense();
            *a_l.x0 = 1234;
            *a_l.x1 = 5678;
        }
        a_
    })
}

#[bench]
fn u64x2_writer_uninit(b: &mut Bencher) {
    b.bytes = 16;
    b.iter(move || {
        let mut a_ = unsafe { U64x2::writer_uninit() };
        {
            let mut a_l = a_.borrow_lense();
            *a_l.x0 = 1234;
            *a_l.x1 = 5678;
        }
        a_
    })
}

make_lense!{PUB, U64x31, U64x31Writer,
    x0: u64,
    x1: u64,
    x2: u64,
    x3: u64,
    x4: u64,
    x5: u64,
    x6: u64,
    x7: u64,
    x8: u64,
    x9: u64,
    x10: u64,
    x11: u64,
    x12: u64,
    x13: u64,
    x14: u64,
    x15: u64,
    x16: u64,
    x17: u64,
    x18: u64,
    x19: u64,
    x20: u64,
    x21: u64,
    x22: u64,
    x23: u64,
    x24: u64,
    x25: u64,
    x26: u64,
    x27: u64,
    x28: u64,
    x29: u64,
    x30: u64,
} unsafe_writer!{U64x31, U64x31Writer}

#[bench]
fn u64x31_reader(b: &mut Bencher) {
    b.bytes = 248;
    let mut a = vec![0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                     ];
    b.iter(|| {
        // Format: [(size, chunk); n] where n is number of chunks
        let mut pos = 0;
        loop {
            let (_, mut a) = a.split_at_mut(pos);
            match U64x31::from_buf(&mut a) {
                Ok((mut a, size)) => { *a.x0 += pos as u64; pos += size },
                Err(LenseError::NothingToParse) => break, // no more to process :)
                Err(LenseError::UnexpectedSize) => break, // Invalid chunk
                Err(LenseError::Incomplete) => break, // Incomplete
            };
        }
        pos
    })
}

#[bench]
fn u64x31_writer_init(b: &mut Bencher) {
    b.bytes = 248;
    b.iter(move || {
        let mut a_ = U64x31::writer();
        {
            let mut a_l = a_.borrow_lense();
            *a_l.x0 = 8130;
            *a_l.x1 = 8095;
            *a_l.x2 = 9842;
            *a_l.x3 = 15474;
            *a_l.x4 = 29015;
            *a_l.x5 = 14200;
            *a_l.x6 = 24582;
            *a_l.x7 = 18487;
            *a_l.x8 = 12011;
            *a_l.x9 = 31747;
            *a_l.x10 = 21740;
            *a_l.x11 = 21135;
            *a_l.x12 = 16288;
            *a_l.x13 = 24138;
            *a_l.x14 = 25480;
            *a_l.x15 = 10901;
            *a_l.x16 = 7979;
            *a_l.x17 = 20033;
            *a_l.x18 = 8983;
            *a_l.x19 = 17450;
            *a_l.x20 = 18682;
            *a_l.x21 = 12055;
            *a_l.x22 = 19986;
            *a_l.x23 = 11670;
            *a_l.x24 = 21643;
            *a_l.x25 = 415;
            *a_l.x26 = 10859;
            *a_l.x27 = 27649;
            *a_l.x28 = 25721;
            *a_l.x29 = 19556;
            *a_l.x30 = 26464;
        }
        a_
    })
}

#[bench]
fn u64x31_writer_uninit(b: &mut Bencher) {
    b.bytes = 248;
    b.iter(move || {
        let mut a_ = unsafe { U64x31::writer_uninit() };
        {
            let mut a_l = a_.borrow_lense();
            *a_l.x0 = 8130;
            *a_l.x1 = 8095;
            *a_l.x2 = 9842;
            *a_l.x3 = 15474;
            *a_l.x4 = 29015;
            *a_l.x5 = 14200;
            *a_l.x6 = 24582;
            *a_l.x7 = 18487;
            *a_l.x8 = 12011;
            *a_l.x9 = 31747;
            *a_l.x10 = 21740;
            *a_l.x11 = 21135;
            *a_l.x12 = 16288;
            *a_l.x13 = 24138;
            *a_l.x14 = 25480;
            *a_l.x15 = 10901;
            *a_l.x16 = 7979;
            *a_l.x17 = 20033;
            *a_l.x18 = 8983;
            *a_l.x19 = 17450;
            *a_l.x20 = 18682;
            *a_l.x21 = 12055;
            *a_l.x22 = 19986;
            *a_l.x23 = 11670;
            *a_l.x24 = 21643;
            *a_l.x25 = 415;
            *a_l.x26 = 10859;
            *a_l.x27 = 27649;
            *a_l.x28 = 25721;
            *a_l.x29 = 19556;
            *a_l.x30 = 26464;
        }
        a_
    })
}
