use alloc::string::String;
use core::fmt;

use crate::builder::FormatBuilder;
use crate::error::FormatError;
use crate::param::{FormatParam, collect_args};
use crate::template::Template;

/// A template string that can be formatted with runtime arguments.
///
/// Implemented for `str`; `String`, `Box<str>`, etc. get it through deref.
/// See the [crate documentation](crate) for the supported placeholder syntax.
///
/// These one-shot methods parse the template on every call; for a template
/// used repeatedly, parse it once with [`Template::parse`](crate::Template::parse).
pub trait Format {
    /// Formats `param` into a new `String`.
    ///
    /// # Panics
    ///
    /// Panics if the template or the arguments are invalid. Use
    /// [`try_format`](Format::try_format) for a fallible version.
    fn format(&self, param: &dyn FormatParam) -> String {
        self.try_format(param)
            .unwrap_or_else(|error| panic!("format failed: {error}"))
    }

    /// Formats `param` into a new `String`.
    ///
    /// # Errors
    ///
    /// Returns [`FormatError::InvalidFormatString`] for a syntactically
    /// invalid template, [`FormatError::InsufficientParameters`] for a
    /// placeholder index beyond the supplied arguments,
    /// [`FormatError::UnsupportedFormatType`] when an argument does not
    /// support the requested format type, and [`FormatError::ExpectedUsize`]
    /// when a dynamic width/precision argument is not an unsigned integer.
    fn try_format(&self, param: &dyn FormatParam) -> Result<String, FormatError>;

    /// Like [`try_format`](Format::try_format), but returns `None` on error.
    fn checked_format(&self, param: &dyn FormatParam) -> Option<String> {
        self.try_format(param).ok()
    }

    /// Formats `param` into `sink`, appending to its current contents.
    ///
    /// # Panics
    ///
    /// Panics if the template or the arguments are invalid, or the sink
    /// rejects a write. Use [`try_write_to`](Format::try_write_to) for a
    /// fallible version.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtformat::Format;
    ///
    /// let mut log = String::from("log: ");
    /// "user {} ({})".write_to(&mut log, &("ada", 7));
    /// assert_eq!(log, "log: user ada (7)");
    /// ```
    fn write_to(&self, sink: &mut dyn fmt::Write, param: &dyn FormatParam) {
        self.try_write_to(sink, param)
            .unwrap_or_else(|error| panic!("format failed: {error}"))
    }

    /// Formats `param` into `sink`, appending to its current contents.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`try_format`](Format::try_format), plus
    /// [`FormatError::WriteFailed`] if the sink rejects a write. Writing
    /// into a `String` never fails.
    fn try_write_to(
        &self,
        sink: &mut dyn fmt::Write,
        param: &dyn FormatParam,
    ) -> Result<(), FormatError>;

    /// Starts a [`FormatBuilder`] for adding arguments one at a time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtformat::Format;
    ///
    /// assert_eq!("{} + {} = {2}".builder().arg(&1).arg(&2).arg(&3).build(), "1 + 2 = 3");
    /// ```
    fn builder<'t>(&'t self) -> FormatBuilder<'t, 'static>;
}

/// Template implementation for string slices. `String`, `Box<str>`, etc.
/// get it through deref.
impl Format for str {
    fn try_format(&self, param: &dyn FormatParam) -> Result<String, FormatError> {
        let mut out = String::with_capacity(self.len());
        self.try_write_to(&mut out, param)?;
        Ok(out)
    }

    fn try_write_to(
        &self,
        sink: &mut dyn fmt::Write,
        param: &dyn FormatParam,
    ) -> Result<(), FormatError> {
        let args = collect_args(param);
        Template::parse(self)?.render(&args, sink)
    }

    fn builder<'t>(&'t self) -> FormatBuilder<'t, 'static> {
        FormatBuilder::new(self)
    }
}
