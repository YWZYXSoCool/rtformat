//! Convenience re-exports of the most commonly used items.
//!
//! # Examples
//!
//! ```
//! use core::fmt;
//! use rtformat::prelude::*;
//!
//! assert_eq!(rformat!("{} + {} = {2}", 1, 2, 3), "1 + 2 = 3");
//!
//! #[derive(Debug, FormatArg)]
//! struct Color(u8, u8, u8);
//!
//! impl fmt::Display for Color {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
//!     }
//! }
//!
//! assert_eq!(rformat!("{}", Color(255, 128, 0)), "#ff8000");
//! ```

pub use crate::{
    Format, FormatArg, FormatBuilder, FormatError, FormatParam, FormatType, Template, rformat,
};
