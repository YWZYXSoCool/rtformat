use alloc::boxed::Box;
use alloc::string::String;
use core::fmt;

/// The trailing type character of a `{...}` placeholder spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatType {
    /// `{}` — default `Display` formatting.
    Display,
    /// `{:?}` (combined with `#`, i.e. `{:#?}`, for pretty output).
    Debug,
    /// `{:x}`
    LowerHex,
    /// `{:X}`
    UpperHex,
    /// `{:o}`
    Octal,
    /// `{:b}`
    Binary,
    /// `{:e}`
    LowerExp,
    /// `{:E}`
    UpperExp,
}

/// A value that can be substituted into a `{}` placeholder.
///
/// Types implementing `Display` and/or `Debug` can use
/// `#[derive(FormatArg)]` instead of writing this impl by hand; the derive
/// adapts to whichever of the two traits is present (`{}` falls back from
/// `Display` to `Debug`). See the
/// [crate documentation](crate#custom-argument-types). When implementing
/// manually, type/format combinations the value does not support (e.g.
/// `{:x}` on a string) should return `Err(fmt::Error)` from [`Self::write`];
/// the engine converts that into
/// [`FormatError::UnsupportedFormatType`](crate::FormatError::UnsupportedFormatType).
///
/// # Examples
///
/// ```
/// use core::fmt;
/// use rtformat::{rformat, Format, FormatArg, FormatType};
///
/// struct Color(u8, u8, u8);
///
/// impl FormatArg for Color {
///     fn write(
///         &self,
///         ty: FormatType,
///         _pretty: bool,
///         _precision: Option<usize>,
///         f: &mut fmt::Formatter<'_>,
///     ) -> fmt::Result {
///         match ty {
///             FormatType::Display => write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2),
///             FormatType::Debug => write!(f, "Color({}, {}, {})", self.0, self.1, self.2),
///             _ => Err(fmt::Error),
///         }
///     }
/// }
///
/// assert_eq!(rformat!("{}", Color(255, 128, 0)), "#ff8000");
/// ```
pub trait FormatArg {
    /// Renders the **bare** value for the given format `ty`.
    ///
    /// Only floating-point precision (digits after the point) is applied here.
    /// Signs, `#` radix prefixes, width, and alignment are all applied by the
    /// engine afterwards — do not apply them here. Precision is ignored for
    /// integers (same as std).
    ///
    /// When `pretty` is `true` and `ty` is [`FormatType::Debug`], render in
    /// the `{:#?}` style.
    ///
    /// # Errors
    ///
    /// Returns `Err(fmt::Error)` if the value does not support the requested
    /// format type. The engine maps this to
    /// [`FormatError::UnsupportedFormatType`](crate::FormatError::UnsupportedFormatType).
    fn write(
        &self,
        ty: FormatType,
        pretty: bool,
        precision: Option<usize>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result;

    /// Whether the value is numeric. Affects the default alignment (right)
    /// and the `+` sign flag.
    fn is_numeric(&self) -> bool {
        false
    }

    /// Whether the value is an integer: all radix types are supported, and
    /// precision is ignored (same as std).
    fn is_integer(&self) -> bool {
        false
    }

    /// The value as a `usize`, used to resolve dynamic width/precision such
    /// as `{:1$}` / `{:.1$}`. Returns `None` for non-unsigned-integer values.
    fn as_usize(&self) -> Option<usize> {
        None
    }
}

macro_rules! impl_format_arg_int {
    ($($t:ty),+ $(,)?) => {$(
        impl FormatArg for $t {
            fn write(
                &self,
                ty: FormatType,
                pretty: bool,
                _precision: Option<usize>,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                match ty {
                    FormatType::Display => fmt::Display::fmt(self, f),
                    FormatType::Debug if pretty => write!(f, "{:#?}", self),
                    FormatType::Debug => fmt::Debug::fmt(self, f),
                    FormatType::LowerHex => fmt::LowerHex::fmt(self, f),
                    FormatType::UpperHex => fmt::UpperHex::fmt(self, f),
                    FormatType::Octal => fmt::Octal::fmt(self, f),
                    FormatType::Binary => fmt::Binary::fmt(self, f),
                    FormatType::LowerExp => fmt::LowerExp::fmt(self, f),
                    FormatType::UpperExp => fmt::UpperExp::fmt(self, f),
                }
            }
            fn is_numeric(&self) -> bool { true }
            fn is_integer(&self) -> bool { true }
            fn as_usize(&self) -> Option<usize> { (*self).try_into().ok() }
        }
    )+};
}
impl_format_arg_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

macro_rules! impl_format_arg_float {
    ($($t:ty),+ $(,)?) => {$(
        impl FormatArg for $t {
            fn write(
                &self,
                ty: FormatType,
                pretty: bool,
                precision: Option<usize>,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                match ty {
                    FormatType::Display => match precision {
                        Some(prec) => write!(f, "{:.prec$}", self, prec = prec),
                        None => fmt::Display::fmt(self, f),
                    },
                    FormatType::Debug if pretty => write!(f, "{:#?}", self),
                    FormatType::Debug => fmt::Debug::fmt(self, f),
                    FormatType::LowerExp => match precision {
                        Some(prec) => write!(f, "{:.prec$e}", self, prec = prec),
                        None => fmt::LowerExp::fmt(self, f),
                    },
                    FormatType::UpperExp => match precision {
                        Some(prec) => write!(f, "{:.prec$E}", self, prec = prec),
                        None => fmt::UpperExp::fmt(self, f),
                    },
                    _ => Err(fmt::Error),
                }
            }
            fn is_numeric(&self) -> bool { true }
        }
    )+};
}
impl_format_arg_float!(f32, f64);

macro_rules! impl_format_arg_text {
    ($($t:ty),+ $(,)?) => {$(
        impl FormatArg for $t {
            fn write(
                &self,
                ty: FormatType,
                pretty: bool,
                _precision: Option<usize>,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                match ty {
                    FormatType::Display => fmt::Display::fmt(self, f),
                    FormatType::Debug if pretty => write!(f, "{:#?}", self),
                    FormatType::Debug => fmt::Debug::fmt(self, f),
                    _ => Err(fmt::Error),
                }
            }
        }
    )+};
}
impl_format_arg_text!(str, String, char, bool);

impl<T: FormatArg + ?Sized> FormatArg for &T {
    fn write(
        &self,
        ty: FormatType,
        pretty: bool,
        precision: Option<usize>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        (**self).write(ty, pretty, precision, f)
    }
    fn is_numeric(&self) -> bool {
        (**self).is_numeric()
    }
    fn is_integer(&self) -> bool {
        (**self).is_integer()
    }
    fn as_usize(&self) -> Option<usize> {
        (**self).as_usize()
    }
}

impl<T: FormatArg + ?Sized> FormatArg for Box<T> {
    fn write(
        &self,
        ty: FormatType,
        pretty: bool,
        precision: Option<usize>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        (**self).write(ty, pretty, precision, f)
    }
    fn is_numeric(&self) -> bool {
        (**self).is_numeric()
    }
    fn is_integer(&self) -> bool {
        (**self).is_integer()
    }
    fn as_usize(&self) -> Option<usize> {
        (**self).as_usize()
    }
}
