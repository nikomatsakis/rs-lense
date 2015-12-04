pub unsafe trait Primitive {}
pub unsafe trait SizedLense {
    fn size() -> usize;
}

// unsigned
unsafe impl Primitive for u8  {}
unsafe impl Primitive for u16 {}
unsafe impl Primitive for u32 {}
unsafe impl Primitive for u64 {}

// signed
unsafe impl Primitive for i8  {}
unsafe impl Primitive for i16 {}
unsafe impl Primitive for i32 {}
unsafe impl Primitive for i64 {}

// floating
unsafe impl Primitive for f32 {}
unsafe impl Primitive for f64 {}
