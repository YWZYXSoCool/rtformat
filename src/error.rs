use core::fmt;

/// Error returned by [`Format::try_format`](crate::Format::try_format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatError {
    /// A placeholder index is beyond the number of supplied arguments.
    InsufficientParameters,
    /// The template is syntactically invalid (unbalanced braces, named
    /// arguments, an illegal spec, ...).
    InvalidFormatString,
    /// The argument does not support the requested format type (e.g. `{:x}`
    /// on a string, or the unsupported `{:p}`).
    UnsupportedFormatType,
    /// The argument referenced by `{:1$}` / `{:.1$}` as a width/precision is
    /// not an unsigned integer.
    ExpectedUsize,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::InsufficientParameters => write!(f, "Insufficient parameters"),
            FormatError::InvalidFormatString => write!(f, "Invalid format string"),
            FormatError::UnsupportedFormatType => {
                write!(f, "Argument does not support the requested format type")
            }
            FormatError::ExpectedUsize => {
                write!(f, "Width/precision argument must be an unsigned integer")
            }
        }
    }
}

impl core::error::Error for FormatError {}
