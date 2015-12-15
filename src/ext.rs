use {Cursor, Result, Ref, Mut};

pub trait DstExt<S, L> where S: Ref {
    type Ret;

    fn set_length(&mut Cursor<S>, u16) -> Result<Self::Ret> where S: Mut;
    fn with_length(&mut Cursor<S>, u16) -> Result<Self::Ret>;
}
