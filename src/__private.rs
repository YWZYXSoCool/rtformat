//! Implementation details of `#[derive(FormatArg)]`. Not public API.
//!
//! Compile-time dispatch via inherent-vs-trait priority: the inherent
//! methods on [`Wrap`] win when their trait bounds hold; otherwise the
//! blanket-implemented fallback traits of the same name take over, so the
//! selection is fully resolved at compile time with no runtime cost.

use core::fmt;

/// Signals "the required trait is not implemented" — distinct from a real
/// formatting failure, so genuine errors still propagate.
#[doc(hidden)]
pub struct FmtAbsent;

/// Wrapper whose inherent methods dispatch to `Display` / `Debug` when the
/// wrapped type implements them.
#[doc(hidden)]
pub struct Wrap<'a, T: ?Sized>(pub &'a T);

impl<T: fmt::Display + ?Sized> Wrap<'_, T> {
    pub fn fmt_display(&self, f: &mut fmt::Formatter<'_>) -> Result<fmt::Result, FmtAbsent> {
        Ok(fmt::Display::fmt(self.0, f))
    }
}

impl<T: fmt::Debug + ?Sized> Wrap<'_, T> {
    pub fn fmt_debug(&self, f: &mut fmt::Formatter<'_>) -> Result<fmt::Result, FmtAbsent> {
        Ok(fmt::Debug::fmt(self.0, f))
    }

    pub fn fmt_debug_pretty(&self, f: &mut fmt::Formatter<'_>) -> Result<fmt::Result, FmtAbsent> {
        Ok(write!(f, "{:#?}", self.0))
    }
}

/// Fallback taking over `{}` when `Display` is not implemented; the derived
/// `write` then tries `Debug` before giving up.
#[doc(hidden)]
pub trait DisplayFallback {
    fn fmt_display(&self, _f: &mut fmt::Formatter<'_>) -> Result<fmt::Result, FmtAbsent> {
        Err(FmtAbsent)
    }
}

impl<T: ?Sized> DisplayFallback for Wrap<'_, T> {}

/// Fallback taking over `{:?}` / `{:#?}` when `Debug` is not implemented.
#[doc(hidden)]
pub trait DebugFallback {
    fn fmt_debug(&self, _f: &mut fmt::Formatter<'_>) -> Result<fmt::Result, FmtAbsent> {
        Err(FmtAbsent)
    }

    fn fmt_debug_pretty(&self, _f: &mut fmt::Formatter<'_>) -> Result<fmt::Result, FmtAbsent> {
        Err(FmtAbsent)
    }
}

impl<T: ?Sized> DebugFallback for Wrap<'_, T> {}
