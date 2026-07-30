use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::arg::{FormatArg, FormatType};
use crate::error::FormatError;
use crate::template::Template;

/// Incrementally formats a template string by adding one argument at a time.
///
/// Created through [`Format::builder`](crate::Format::builder); finish with
/// [`build`](FormatBuilder::build) (panicking) or
/// [`try_build`](FormatBuilder::try_build) (fallible), or with
/// [`build_to`](FormatBuilder::build_to) /
/// [`try_build_to`](FormatBuilder::try_build_to) to write into an existing
/// [`core::fmt::Write`] sink.
///
/// # Examples
///
/// ```
/// use rtformat::Format;
///
/// let s = "{} + {} = {2}".builder().arg(&1).arg(&2).arg(&3).build();
/// assert_eq!(s, "1 + 2 = 3");
///
/// let width = 5;
/// let s = "{:1$}".builder().arg("ab").arg(&width).build();
/// assert_eq!(s, "ab   ");
/// ```
pub struct FormatBuilder<'template, 'arg> {
    template: &'template str,
    slots: Vec<ArgSlot<'arg>>,
}

impl<'template, 'arg> FormatBuilder<'template, 'arg> {
    pub(crate) fn new(template: &'template str) -> Self {
        Self {
            template,
            slots: Vec::new(),
        }
    }

    /// Appends the next positional argument.
    ///
    /// Arguments are indexed in call order; explicit indices (`{1}`) and
    /// reused placeholders (`{0} {0}`) refer to this order. Accepts any
    /// reference to a [`FormatArg`] type, including bare string literals
    /// (`.arg("text")`).
    pub fn arg<A: FormatArgRef<'arg>>(mut self, arg: A) -> Self {
        self.slots.push(arg.into_slot());
        self
    }

    /// Appends multiple positional arguments from an iterator.
    ///
    /// A convenience for repeated [`arg`](FormatBuilder::arg) calls;
    /// arguments are indexed in iteration order. Because iterators are
    /// homogeneous, every item must share one concrete type — to mix
    /// types, chain [`arg`](FormatBuilder::arg) calls instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtformat::Format;
    ///
    /// let s = "{} {} {}".builder().args(["a", "b", "c"]).build();
    /// assert_eq!(s, "a b c");
    ///
    /// // Any `IntoIterator` works, e.g. a `Vec` of values:
    /// let values = vec![1, 2, 3];
    /// let s = "{} + {} = {2}".builder().args(values.iter()).build();
    /// assert_eq!(s, "1 + 2 = 3");
    /// ```
    pub fn args<I>(mut self, args: I) -> Self
    where
        I: IntoIterator,
        I::Item: FormatArgRef<'arg>,
    {
        self.slots.extend(args.into_iter().map(|a| a.into_slot()));
        self
    }

    /// Consumes the builder and formats the template.
    ///
    /// # Panics
    ///
    /// Panics if the template or the arguments are invalid. Use
    /// [`try_build`](FormatBuilder::try_build) for a fallible version.
    pub fn build(self) -> String {
        self.try_build()
            .unwrap_or_else(|error| panic!("format failed: {error}"))
    }

    /// Consumes the builder and formats the template.
    ///
    /// # Errors
    ///
    /// Returns the same errors as
    /// [`Format::try_format`](crate::Format::try_format).
    pub fn try_build(self) -> Result<String, FormatError> {
        let mut out = String::with_capacity(self.template.len());
        self.try_build_to(&mut out)?;
        Ok(out)
    }

    /// Consumes the builder and formats the template into `sink`,
    /// appending to its current contents.
    ///
    /// # Panics
    ///
    /// Panics if the template or the arguments are invalid, or the sink
    /// rejects a write. Use [`try_build_to`](FormatBuilder::try_build_to)
    /// for a fallible version.
    ///
    /// # Examples
    ///
    /// ```
    /// use rtformat::Format;
    ///
    /// let mut buf = String::from("sum: ");
    /// "{} + {}".builder().arg(&1).arg(&2).build_to(&mut buf);
    /// assert_eq!(buf, "sum: 1 + 2");
    /// ```
    pub fn build_to(self, sink: &mut dyn fmt::Write) {
        self.try_build_to(sink)
            .unwrap_or_else(|error| panic!("format failed: {error}"))
    }

    /// Consumes the builder and formats the template into `sink`,
    /// appending to its current contents.
    ///
    /// # Errors
    ///
    /// Returns the same errors as
    /// [`Format::try_format`](crate::Format::try_format), plus
    /// [`FormatError::WriteFailed`] if the sink rejects a write.
    pub fn try_build_to(self, sink: &mut dyn fmt::Write) -> Result<(), FormatError> {
        let args: Vec<&dyn FormatArg> = self
            .slots
            .iter()
            .map(|slot| slot as &dyn FormatArg)
            .collect();
        Template::parse(self.template)?.render(&args, sink)
    }
}

/// A type that can be passed to [`FormatBuilder::arg`].
///
/// Implemented for `&'a T` where `T: FormatArg`, and additionally for
/// `&'a str` so that bare string literals are accepted directly.
pub trait FormatArgRef<'a> {
    #[doc(hidden)]
    fn into_slot(self) -> ArgSlot<'a>;
}

impl<'a, T: FormatArg + 'a> FormatArgRef<'a> for &'a T {
    fn into_slot(self) -> ArgSlot<'a> {
        ArgSlot::Dyn(self)
    }
}

impl<'a> FormatArgRef<'a> for &'a str {
    fn into_slot(self) -> ArgSlot<'a> {
        ArgSlot::Str(self)
    }
}

/// Storage for one builder argument. `Str` exists because `&str` cannot be
/// coerced to `&dyn FormatArg` (`str` is not `Sized`).
#[doc(hidden)]
pub enum ArgSlot<'a> {
    Dyn(&'a dyn FormatArg),
    Str(&'a str),
}

impl FormatArg for ArgSlot<'_> {
    fn write(
        &self,
        ty: FormatType,
        pretty: bool,
        precision: Option<usize>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ArgSlot::Dyn(arg) => arg.write(ty, pretty, precision, f),
            ArgSlot::Str(arg) => arg.write(ty, pretty, precision, f),
        }
    }

    fn is_numeric(&self) -> bool {
        match self {
            ArgSlot::Dyn(arg) => arg.is_numeric(),
            ArgSlot::Str(arg) => arg.is_numeric(),
        }
    }

    fn is_integer(&self) -> bool {
        match self {
            ArgSlot::Dyn(arg) => arg.is_integer(),
            ArgSlot::Str(arg) => arg.is_integer(),
        }
    }

    fn as_usize(&self) -> Option<usize> {
        match self {
            ArgSlot::Dyn(arg) => arg.as_usize(),
            ArgSlot::Str(arg) => arg.as_usize(),
        }
    }
}
