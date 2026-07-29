//! Runtime string formatting with `std::fmt`-compatible placeholder syntax.
//!
//! - `{}` implicit positional arguments; `{0}` / `{1}` explicit indices (reusable)
//! - Types: `{:?}` `{:#?}` `{:x}` `{:X}` `{:o}` `{:b}` `{:e}` `{:E}`
//! - Alignment and fill: `{:<10}` `{:>10}` `{:^10}` `{:_>10}`
//! - Sign / radix prefix / zero padding: `{:+}` `{:#x}` `{:010}`
//! - Width and precision: `{:.3}` `{:1$}` `{:.1$}` (`n$` takes the width/precision from argument `n`)
//! - `{{` / `}}` escapes
//! - Format into any `core::fmt::Write` sink; precompiled templates via [`Template`]
//!
//! Not supported: named arguments `{name}`, pointers `{:p}`, hexadecimal debug `{x?}` / `{X?}`.
//!
//! This crate is `#![no_std]`-compatible and requires `alloc`.
//!
//! # Custom argument types
//!
//! Types implementing `Display` and/or `Debug` can derive [`FormatArg`].
//! The derive adapts: `{}` prefers `Display` and falls back to `Debug`,
//! while `{:?}` / `{:#?}` require `Debug`:
//!
//! ```
//! use core::fmt;
//! use rtformat::{rformat, Format, FormatArg};
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
//!
//! For full control over each format type, implement [`FormatArg`] manually
//! instead.
//!
//! # Examples
//!
//! ```
//! use rtformat::{rformat, Format};
//!
//! assert_eq!(rformat!("Hello {}!", "world"), "Hello world!");
//! assert_eq!("{} + {} = {2}".format(&(1, 2, 3)), "1 + 2 = 3");
//! assert_eq!(rformat!("{:#010x}", 255), "0x000000ff");
//! ```
//!
//! # Builder API
//!
//! Templates can also be formatted incrementally with
//! [`FormatBuilder`], adding one argument per call:
//!
//! ```
//! use rtformat::Format;
//!
//! let s = "{} + {} = {2}".builder().arg(&1).arg(&2).arg(&3).build();
//! assert_eq!(s, "1 + 2 = 3");
//! ```
//!
//! # Precompiled templates
//!
//! The one-shot APIs parse the template on every call. [`Template`] parses
//! once, formats many times, and can also write into any
//! [`core::fmt::Write`] sink (appending, not overwriting):
//!
//! ```
//! use rtformat::Template;
//!
//! let tpl = Template::parse("{} + {} = {2}")?;
//! assert_eq!(tpl.format(&(1, 2, 3)), "1 + 2 = 3");
//!
//! let mut log = String::from("log: ");
//! tpl.try_write_to(&mut log, &(1, 2, 3))?;
//! assert_eq!(log, "log: 1 + 2 = 3");
//! # Ok::<(), rtformat::FormatError>(())
//! ```
//!
//! All commonly used items are also available through [`prelude`]:
//!
//! ```
//! use rtformat::prelude::*;
//!
//! assert_eq!(rformat!("Hello {}!", "world"), "Hello world!");
//! ```

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

mod arg;
mod builder;
mod engine;
mod error;
mod format;
mod param;
mod spec;
mod template;

#[doc(hidden)]
pub mod __private;

pub mod prelude;

#[cfg(test)]
mod std_compat;

#[cfg(test)]
mod tests;

extern crate self as rtformat;

pub use arg::{FormatArg, FormatType};
pub use builder::{FormatArgRef, FormatBuilder};
pub use error::FormatError;
pub use format::Format;
pub use param::FormatParam;
pub use template::Template;

#[doc(inline)]
pub use rtformat_derive::FormatArg;

/// Formats arguments into a template string at runtime.
///
/// Convenience wrapper around [`Format::format`]: packs the arguments into a
/// tuple and calls `format`.
///
/// # Panics
///
/// Panics if the template or the arguments are invalid. Use
/// [`Format::try_format`] if failure is possible.
///
/// # Examples
///
/// ```
/// use rtformat::{rformat, Format};
///
/// assert_eq!(rformat!("{0} + {1} = {2}", 1, 2, 3), "1 + 2 = 3");
/// assert_eq!(rformat!("{:.2}", 3.14159), "3.14");
/// ```
#[macro_export]
macro_rules! rformat {
    ($fmt: expr $(, $params: expr)* $(,)?) => {
        $fmt.format(&($($params,)*))
    };
}
