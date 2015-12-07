use {Cursor, Result};

pub trait DstExt<'a, S, L> {
    type Ret;
    fn with_length(&mut Cursor<S>, u16) -> Result<Self::Ret>;
}
