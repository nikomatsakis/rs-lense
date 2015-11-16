
//! Macro generated struct `Alice` and enum `Bob`
//!
//! ```
//! # #![allow(dead_code)]
//! # #[macro_use] extern crate lense;
//! mk_lense_struct!{pub struct Alice:
//!     a:  u8,
//!     /// Documentation is optional
//!     pub bc: (u8, u16),
//!     d:  u32,
//!     e:  u64,
//! }
//! # fn main() {}
//! ```
//!
//! ```
//! # #![allow(dead_code)]
//! # #[macro_use] extern crate lense;
//! #[cfg(feature = "experimental_lense_enums")]
//! mk_lense_struct!{pub enum Bob:
//!     U8(u8),
//!     /// Documentation is optional
//!     U16(u16),
//!     U32(u32),
//! }
//! # fn main() {}
//! ```

#![allow(dead_code)]

use Lense;

mk_lense_struct!{
    pub struct Alice:
        a:  u8,
        /// Documentation is optional
        pub bc: (u8, u16),
        d:  u32,
        e:  u64,
}

#[cfg(feature = "experimental_lense_enums")]
mk_lense_struct!{
    /// Documenting things is good
    pub enum Bob:
        U8(u8),
        /// although, doucmentation is optional
        U16(u16),
        U32(u32),
}
