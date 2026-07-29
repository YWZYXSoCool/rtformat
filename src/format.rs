use alloc::string::String;
use alloc::vec::Vec;

use crate::arg::FormatArg;
use crate::builder::FormatBuilder;
use crate::engine::format_with_args;
use crate::error::FormatError;
use crate::param::FormatParam;

/// A template string that can be formatted with runtime arguments.
///
/// Implemented for `str`; `String`, `Box<str>`, etc. get it through deref.
/// See the [crate documentation](crate) for the supported placeholder syntax.
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
        let mut args: Vec<&dyn FormatArg> = Vec::new();
        param.visit(&mut |arg| args.push(arg));
        format_with_args(self, &args)
    }

    fn builder<'t>(&'t self) -> FormatBuilder<'t, 'static> {
        FormatBuilder::new(self)
    }
}
