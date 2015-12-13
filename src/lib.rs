//! Lense: Result > Panic
//! =====================
//!
//! A faster lense implementation performing manual bounds checks to avoid unwinding code when
//! using `slice.split_at{,_mut}`. Now with 3-raw pointers for a cursor, lenses are significantly
//! faster! [insert benchmarks]
//!
//! The current 3-ptr cursor makes it easier to cleanly handle DSTs and this may be reduced to
//! 2-ptr if it is proven that we never need to go backwards. The raw pointers remain faster than
//! hacky slices.
//!
//! Currently implemented types:
//!
//! - Primitive numbers: u8 i8 u16 i16 u32 i32 f32 u64 i64 f64
//! - Primitive slices: [Prim]
//! - Lense composed tuples and arrays
//! - Vectors of sized lenses
//!
//! Old overhead removed: assert! and thread rewinding on panic (Deoptimizations)
//! New overhead added: try! *takes longer to compile*
//!
//! Performance increase: ~5x
//!
//! Prefer primitive slices over composed vectors
//!
//! Prefer external partitioning for DST associated data, allowing the base to be sized
//!
//! Remaining features to implement or port
//! ---------------------------------------
//!
//! - Ideal struct, enum, and union DSL
//!   - Fairly complicated macro inbound
//! - Bitflags for booleans and to condense two-state union tags like Option<T>, Result<T, F>

mod aligned;
mod cur;

/// Primitive numeric types
mod prim;
/// Arrays and tuples
mod lense;
/// Vectors/iterators
mod vec;
/// Primitive slice
mod slice;

/// Tag iterator (includes Option and Result types)
mod tag;

/// DstExt
mod ext;
/// Endian
mod endian;

/// Macro to assist with creating lense structs/enums
#[macro_use]
mod dsl;

pub use aligned::Aligned;
pub use cur::{Cursor, RefMut};
pub use endian::Endian;
pub use lense::Lense;
pub use prim::{AlignedLense, Primitive, SizedLense};
pub use ext::DstExt;
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
