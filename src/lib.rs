//! Lense-1.0.0
//! ===========
//!
//!

mod aligned;
mod cur;

mod prim;
mod lense;
mod vec;
mod slice;

mod tag;

mod ext;
mod endian;

#[macro_use]
mod dsl;

pub use aligned::Aligned;
pub use cur::{Cursor, RefMut};
pub use endian::Endian;
pub use ext::DstExt;
pub use lense::Lense;
pub use prim::{AlignedLense, Primitive, SizedLense};
pub use tag::{BoolGuard, Tag, padded};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Length check failed
    ExpectedBytes(usize),
    /// Length check for LenseTag failed
    ExpectedBits(usize),
    /// Enum or union tag is invalid
    InvalidTag(usize),
    /// No tag was defined, but the lense type attempted to use one
    NoTagDefined,
}

/// Reference to a type which implements Deref<Target=[u8]>, while not containing any lifetime in
/// this trait, one must be present on the Self type.
///
/// Unsafe if no lifetime is provided
pub unsafe trait Ref: ::std::ops::Deref<Target=[u8]> { }
unsafe impl<'a> Ref for &'a [u8] { }
unsafe impl<'a> Ref for &'a mut [u8] { }

/// Assert that S is mutable
pub trait Mut: Ref + ::std::ops::DerefMut { }
impl<'a> Mut for &'a mut [u8] { }
