use Primitive;

// Generic trait to call respective .to_le functions on the primitive types
pub trait Endian: Primitive {
    fn handle(self) -> Self;
}

macro_rules! impls {
    ($($t:ident)*) => {$(
        impl Endian for $t {
            fn handle(self) -> Self { self.to_le() }
        }
    )*}
}

impls!{
    u8 u16 u32 u64
    i8 i16 i32 i64
}
