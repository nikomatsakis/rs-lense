use std::ops;
use {Cursor, Result, SizedLense};

pub trait DstExt<S, L>
    where S: ops::Deref
{
    type Ret;
    fn set_length(&mut Cursor<S>, u16) -> Result<Self::Ret>
        where S: ops::DerefMut;
    fn with_length(&mut Cursor<S>, u16) -> Result<Self::Ret>;
}
