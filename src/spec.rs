use alloc::string::String;
use alloc::vec::Vec;

use crate::arg::{FormatArg, FormatType};
use crate::error::FormatError;

/// Alignment of the rendered value within its width.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Align {
    Left,
    Center,
    Right,
}

/// Width/precision: either a literal, or an `n$` argument reference that has
/// not yet been resolved at parse time.
#[derive(Clone, Copy)]
enum Count {
    Lit(usize),
    Param(usize),
}

/// The parsed spec, before `n$` counts have been resolved against the arguments.
#[derive(Clone, Copy)]
pub(crate) struct RawSpec {
    fill: char,
    align: Option<Align>,
    sign_plus: bool,
    alternate: bool,
    zero: bool,
    width: Option<Count>,
    precision: Option<Count>,
    ty: FormatType,
}

/// The fully resolved spec used for rendering.
#[derive(Clone, Copy)]
pub(crate) struct Spec {
    pub(crate) fill: char,
    pub(crate) align: Option<Align>,
    pub(crate) sign_plus: bool,
    pub(crate) alternate: bool,
    pub(crate) zero: bool,
    pub(crate) width: Option<usize>,
    pub(crate) precision: Option<usize>,
    pub(crate) ty: FormatType,
}

impl Spec {
    /// `{:#?}`: pretty-printed Debug.
    pub(crate) fn pretty(self) -> bool {
        self.alternate && self.ty == FormatType::Debug
    }
}

impl RawSpec {
    /// Resolves `n$` width/precision counts against the actual arguments.
    ///
    /// # Errors
    ///
    /// Returns [`FormatError::InsufficientParameters`] if a referenced index
    /// is out of range, or [`FormatError::ExpectedUsize`] if the referenced
    /// argument is not an unsigned integer.
    pub(crate) fn resolve(self, args: &[&dyn FormatArg]) -> Result<Spec, FormatError> {
        fn resolve_count(
            c: Option<Count>,
            args: &[&dyn FormatArg],
        ) -> Result<Option<usize>, FormatError> {
            match c {
                None => Ok(None),
                Some(Count::Lit(n)) => Ok(Some(n)),
                Some(Count::Param(i)) => {
                    let arg = args.get(i).ok_or(FormatError::InsufficientParameters)?;
                    Ok(Some(arg.as_usize().ok_or(FormatError::ExpectedUsize)?))
                }
            }
        }
        Ok(Spec {
            fill: self.fill,
            align: self.align,
            sign_plus: self.sign_plus,
            alternate: self.alternate,
            zero: self.zero,
            width: resolve_count(self.width, args)?,
            precision: resolve_count(self.precision, args)?,
            ty: self.ty,
        })
    }
}

/// Parses the inside of a `{...}` placeholder (without the braces) and
/// returns `(explicit index, spec)`. A `None` index means an implicit
/// positional argument.
///
/// # Errors
///
/// Returns [`FormatError::InvalidFormatString`] for named arguments or an
/// unparseable spec, and propagates errors from [`parse_spec`].
pub(crate) fn parse_placeholder(inner: &str) -> Result<(Option<usize>, RawSpec), FormatError> {
    let (arg_part, spec_part) = match inner.find(':') {
        Some(i) => (&inner[..i], &inner[i + 1..]),
        None => (inner, ""),
    };

    let index = if arg_part.is_empty() {
        None
    } else if arg_part.bytes().all(|b| b.is_ascii_digit()) {
        Some(parse_usize(arg_part).ok_or(FormatError::InvalidFormatString)?)
    } else {
        return Err(FormatError::InvalidFormatString);
    };

    Ok((index, parse_spec(spec_part)?))
}

/// Parses a non-empty ASCII digit string into a `usize`.
fn parse_usize(s: &str) -> Option<usize> {
    let mut n = 0usize;
    for b in s.bytes() {
        n = n.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(n)
}

/// Parses a format spec with the grammar
/// `[[fill]align][sign]['#']['0'][width]['.' precision][type]`.
///
/// # Errors
///
/// Returns [`FormatError::InvalidFormatString`] for malformed input (e.g. a
/// dangling `$`, an empty precision), and
/// [`FormatError::UnsupportedFormatType`] for unknown or unsupported type
/// characters (e.g. `p`, `x?`).
fn parse_spec(s: &str) -> Result<RawSpec, FormatError> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut spec = RawSpec {
        fill: ' ',
        align: None,
        sign_plus: false,
        alternate: false,
        zero: false,
        width: None,
        precision: None,
        ty: FormatType::Display,
    };

    fn align_of(c: char) -> Option<Align> {
        match c {
            '<' => Some(Align::Left),
            '^' => Some(Align::Center),
            '>' => Some(Align::Right),
            _ => None,
        }
    }

    if let Some(&c) = chars.get(i) {
        if let Some(a) = align_of(c) {
            spec.align = Some(a);
            i += 1;
        } else if let Some(&c2) = chars.get(i + 1)
            && let Some(a) = align_of(c2)
        {
            spec.fill = c;
            spec.align = Some(a);
            i += 2;
        }
    }

    match chars.get(i) {
        Some('+') => {
            spec.sign_plus = true;
            i += 1;
        }
        Some('-') => i += 1,
        _ => {}
    }

    if chars.get(i) == Some(&'#') {
        spec.alternate = true;
        i += 1;
    }

    if chars.get(i) == Some(&'0') {
        spec.zero = true;
        i += 1;
    }

    if let Some((count, consumed)) = parse_count(&chars[i..])? {
        spec.width = Some(count);
        i += consumed;
    }

    if chars.get(i) == Some(&'.') {
        i += 1;
        match parse_count(&chars[i..])? {
            Some((count, consumed)) => {
                spec.precision = Some(count);
                i += consumed;
            }
            None => return Err(FormatError::InvalidFormatString),
        }
    }

    let ty: String = chars[i..].iter().collect();
    if ty.contains('$') {
        return Err(FormatError::InvalidFormatString);
    }
    spec.ty = match ty.as_str() {
        "" => FormatType::Display,
        "?" => FormatType::Debug,
        "x" => FormatType::LowerHex,
        "X" => FormatType::UpperHex,
        "o" => FormatType::Octal,
        "b" => FormatType::Binary,
        "e" => FormatType::LowerExp,
        "E" => FormatType::UpperExp,
        _ => return Err(FormatError::UnsupportedFormatType),
    };

    Ok(spec)
}

/// Parses a literal number or an `n$` argument reference; returns `None` if
/// the input does not start with a digit.
///
/// # Errors
///
/// Returns [`FormatError::InvalidFormatString`] if the number overflows
/// `usize`.
fn parse_count(chars: &[char]) -> Result<Option<(Count, usize)>, FormatError> {
    if !chars.first().is_some_and(|c| c.is_ascii_digit()) {
        return Ok(None);
    }
    let mut n = 0usize;
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        match c.to_digit(10) {
            Some(d) => {
                n = n
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(d as usize))
                    .ok_or(FormatError::InvalidFormatString)?;
                i += 1;
            }
            None => break,
        }
    }
    if chars.get(i) == Some(&'$') {
        Ok(Some((Count::Param(n), i + 1)))
    } else {
        Ok(Some((Count::Lit(n), i)))
    }
}
