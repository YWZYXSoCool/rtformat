use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::arg::FormatArg;
use crate::engine::render_arg;
use crate::error::FormatError;
use crate::param::{FormatParam, collect_args};
use crate::spec::{RawSpec, parse_placeholder};

/// One piece of a parsed [`Template`].
#[derive(Clone, Debug, PartialEq)]
enum Segment<'t> {
    /// Verbatim text copied through to the output.
    Literal(&'t str),
    /// One literal `{` or `}` originating from a `{{` / `}}` escape.
    Escaped(char),
    /// One `{...}` placeholder. The positional index is fully resolved at
    /// parse time (implicit counters are materialized); only `n$`
    /// width/precision counts still await the actual arguments.
    Placeholder { index: usize, spec: RawSpec },
}

/// A template string parsed once and formattable many times.
///
/// Created through [`Template::parse`]. Syntax and error semantics match
/// the one-shot APIs ([`rformat!`](crate::rformat),
/// [`Format`](crate::Format)), except that template syntax errors surface
/// at parse time, while argument errors surface at format time.
///
/// # Examples
///
/// ```
/// use rtformat::Template;
///
/// let tpl = Template::parse("{} + {} = {2}")?;
/// assert_eq!(tpl.format(&(1, 2, 3)), "1 + 2 = 3");
///
/// // The same parsed template can be reused with different arguments:
/// assert_eq!(tpl.format(&("a", "b", "ab")), "a + b = ab");
///
/// // Output can also go to any `core::fmt::Write` sink, appending:
/// let mut log = String::from("log: ");
/// tpl.try_write_to(&mut log, &(1, 2, 3))?;
/// assert_eq!(log, "log: 1 + 2 = 3");
/// # Ok::<(), rtformat::FormatError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Template<'t> {
    source: &'t str,
    segments: Vec<Segment<'t>>,
    min_args: usize,
}

impl<'t> Template<'t> {
    /// Parses `template` into a reusable form.
    ///
    /// Literal segments borrow from `template` — no text is copied.
    ///
    /// # Errors
    ///
    /// Returns [`FormatError::InvalidFormatString`] for unbalanced braces,
    /// named arguments, or an illegal spec, and
    /// [`FormatError::UnsupportedFormatType`] for unknown or unsupported
    /// type characters. Argument-count and value errors cannot be detected
    /// until format time.
    pub fn parse(template: &'t str) -> Result<Self, FormatError> {
        let mut segments: Vec<Segment<'t>> = Vec::new();
        let mut min_args = 0usize;
        let mut implicit = 0usize;
        let mut literal_start = 0usize;
        let mut i = 0usize;
        let bytes = template.as_bytes();

        macro_rules! flush_literal {
            ($end:expr) => {
                if literal_start < $end {
                    segments.push(Segment::Literal(&template[literal_start..$end]));
                }
            };
        }

        while i < bytes.len() {
            match bytes[i] {
                b'{' if bytes.get(i + 1) == Some(&b'{') => {
                    flush_literal!(i);
                    segments.push(Segment::Escaped('{'));
                    i += 2;
                    literal_start = i;
                }
                b'{' => {
                    flush_literal!(i);
                    let inner_start = i + 1;
                    let close = template[inner_start..]
                        .find('}')
                        .map(|p| inner_start + p)
                        .ok_or(FormatError::InvalidFormatString)?;
                    let (index, spec) = parse_placeholder(&template[inner_start..close])?;
                    let index = match index {
                        Some(n) => n,
                        None => {
                            let n = implicit;
                            implicit += 1;
                            n
                        }
                    };
                    min_args = min_args.max(index + 1);
                    if let Some(p) = spec.max_param_ref() {
                        min_args = min_args.max(p + 1);
                    }
                    segments.push(Segment::Placeholder { index, spec });
                    i = close + 1;
                    literal_start = i;
                }
                b'}' if bytes.get(i + 1) == Some(&b'}') => {
                    flush_literal!(i);
                    segments.push(Segment::Escaped('}'));
                    i += 2;
                    literal_start = i;
                }
                b'}' => return Err(FormatError::InvalidFormatString),
                _ => i += 1,
            }
        }
        flush_literal!(bytes.len());

        Ok(Self {
            source: template,
            segments,
            min_args,
        })
    }

    /// The minimum number of arguments this template references (counting
    /// placeholder indices and `n$` width/precision references).
    pub fn arity(&self) -> usize {
        self.min_args
    }

    /// Formats `param` into a new `String`.
    ///
    /// # Panics
    ///
    /// Panics if the arguments are invalid. Use
    /// [`try_format`](Template::try_format) for a fallible version.
    pub fn format(&self, param: &dyn FormatParam) -> String {
        self.try_format(param)
            .unwrap_or_else(|error| panic!("format failed: {error}"))
    }

    /// Formats `param` into a new `String`.
    ///
    /// # Errors
    ///
    /// Returns the same argument errors as
    /// [`Format::try_format`](crate::Format::try_format).
    pub fn try_format(&self, param: &dyn FormatParam) -> Result<String, FormatError> {
        let mut out = String::with_capacity(self.source.len());
        self.try_write_to(&mut out, param)?;
        Ok(out)
    }

    /// Formats `param` into `sink`, appending to its current contents.
    ///
    /// # Panics
    ///
    /// Panics if the arguments are invalid or the sink rejects a write.
    /// Use [`try_write_to`](Template::try_write_to) for a fallible version.
    pub fn write_to(&self, sink: &mut dyn fmt::Write, param: &dyn FormatParam) {
        self.try_write_to(sink, param)
            .unwrap_or_else(|error| panic!("format failed: {error}"))
    }

    /// Formats `param` into `sink`, appending to its current contents.
    ///
    /// # Errors
    ///
    /// Returns the same argument errors as
    /// [`Format::try_format`](crate::Format::try_format), plus
    /// [`FormatError::WriteFailed`] if the sink rejects a write.
    pub fn try_write_to(
        &self,
        sink: &mut dyn fmt::Write,
        param: &dyn FormatParam,
    ) -> Result<(), FormatError> {
        let args = collect_args(param);
        self.render(&args, sink)
    }

    /// Renders the parsed segments against `args` into `w`. A single
    /// scratch buffer is reused across placeholders, so a call allocates
    /// at most once for the value payloads regardless of template length.
    pub(crate) fn render(
        &self,
        args: &[&dyn FormatArg],
        w: &mut dyn fmt::Write,
    ) -> Result<(), FormatError> {
        if args.len() < self.min_args {
            return Err(FormatError::InsufficientParameters);
        }
        let mut scratch = String::new();
        for segment in &self.segments {
            match segment {
                Segment::Literal(text) => note_sink(w.write_str(text))?,
                Segment::Escaped(c) => note_sink(w.write_char(*c))?,
                Segment::Placeholder { index, spec } => {
                    let arg = args
                        .get(*index)
                        .ok_or(FormatError::InsufficientParameters)?;
                    render_arg(arg, spec.resolve(args)?, w, &mut scratch)?;
                }
            }
        }
        Ok(())
    }
}

/// Maps a write rejected by the output sink to [`FormatError::WriteFailed`].
fn note_sink(result: fmt::Result) -> Result<(), FormatError> {
    result.map_err(|_| FormatError::WriteFailed)
}
