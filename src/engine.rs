use core::fmt;
use core::fmt::Write as _;

use crate::arg::{FormatArg, FormatType};
use crate::error::FormatError;
use crate::spec::{parse_placeholder, Align, Spec};

/// Display adapter that hands the bare value to [`FormatArg::write`].
/// The `Formatter` it receives carries no flags, guaranteeing the core
/// output is unadorned.
struct Core<'a> {
    arg: &'a dyn FormatArg,
    spec: Spec,
}

impl fmt::Display for Core<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.arg
            .write(self.spec.ty, self.spec.pretty(), self.spec.precision, f)
    }
}

/// The `#` radix prefix for each integer format type.
fn radix_prefix(ty: FormatType) -> &'static str {
    match ty {
        FormatType::LowerHex => "0x",
        FormatType::UpperHex => "0X",
        FormatType::Octal => "0o",
        FormatType::Binary => "0b",
        _ => "",
    }
}

/// Renders one argument according to `spec` and appends it to `out`.
///
/// # Errors
///
/// Returns [`FormatError::UnsupportedFormatType`] if the argument does not
/// support the requested format type.
fn render_arg(arg: &dyn FormatArg, spec: Spec, out: &mut String) -> Result<(), FormatError> {
    let mut payload = String::new();
    write!(payload, "{}", Core { arg, spec }).map_err(|_| FormatError::UnsupportedFormatType)?;

    if !arg.is_numeric() {
        if let Some(p) = spec.precision {
            if payload.chars().count() > p {
                payload = payload.chars().take(p).collect();
            }
        }
        pad_width(arg, spec, payload, out);
        return Ok(());
    }

    let (sign, mut digits) = match payload.strip_prefix('-') {
        Some(rest) => ("-", rest.to_owned()),
        None => {
            if spec.sign_plus {
                ("+", payload)
            } else {
                ("", payload)
            }
        }
    };

    if arg.is_integer() {
        if let Some(p) = spec.precision {
            if digits.len() < p {
                digits = "0".repeat(p - digits.len()) + &digits;
            }
        }
    }

    let prefix = if spec.alternate {
        radix_prefix(spec.ty)
    } else {
        ""
    };

    let zero_fill = spec.zero
        && spec.align.is_none()
        && !(arg.is_integer() && spec.precision.is_some());

    if zero_fill {
        if let Some(width) = spec.width {
            let cur = sign.len() + prefix.len() + digits.len();
            if cur < width {
                digits = "0".repeat(width - cur) + &digits;
            }
        }
        out.push_str(sign);
        out.push_str(prefix);
        out.push_str(&digits);
        return Ok(());
    }

    let mut total = String::new();
    total.push_str(sign);
    total.push_str(prefix);
    total.push_str(&digits);
    pad_width(arg, spec, total, out);
    Ok(())
}

/// Pads `total` to the spec width. Default alignment: right for numerics,
/// left otherwise (same as std).
fn pad_width(arg: &dyn FormatArg, spec: Spec, total: String, out: &mut String) {
    let Some(width) = spec.width else {
        out.push_str(&total);
        return;
    };
    let len = total.chars().count();
    if len >= width {
        out.push_str(&total);
        return;
    }
    let pad = width - len;
    let align = spec.align.unwrap_or(if arg.is_numeric() {
        Align::Right
    } else {
        Align::Left
    });
    let fill = spec.fill;
    match align {
        Align::Left => {
            out.push_str(&total);
            out.extend(core::iter::repeat_n(fill, pad));
        }
        Align::Right => {
            out.extend(core::iter::repeat_n(fill, pad));
            out.push_str(&total);
        }
        Align::Center => {
            let left = pad / 2;
            out.extend(core::iter::repeat_n(fill, left));
            out.push_str(&total);
            out.extend(core::iter::repeat_n(fill, pad - left));
        }
    }
}

/// The formatting engine: walks the template, resolves each placeholder
/// against `args`, and renders it.
///
/// # Errors
///
/// Returns [`FormatError::InvalidFormatString`] for unbalanced braces or an
/// invalid placeholder, [`FormatError::InsufficientParameters`] for an index
/// beyond `args`, and propagates errors from
/// [`RawSpec::resolve`](crate::spec::RawSpec::resolve) and [`render_arg`].
pub(crate) fn format_with_args(
    template: &str,
    args: &[&dyn FormatArg],
) -> Result<String, FormatError> {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let mut implicit = 0usize;

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    out.push('{');
                    continue;
                }
                let mut inner = String::new();
                let mut closed = false;
                for d in chars.by_ref() {
                    if d == '}' {
                        closed = true;
                        break;
                    }
                    inner.push(d);
                }
                if !closed {
                    return Err(FormatError::InvalidFormatString);
                }
                let (index, raw) = parse_placeholder(&inner)?;
                let idx = match index {
                    Some(n) => n,
                    None => {
                        let n = implicit;
                        implicit += 1;
                        n
                    }
                };
                let arg = *args.get(idx).ok_or(FormatError::InsufficientParameters)?;
                let spec = raw.resolve(args)?;
                render_arg(arg, spec, &mut out)?;
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    out.push('}');
                    continue;
                }
                return Err(FormatError::InvalidFormatString);
            }
            c => out.push(c),
        }
    }
    Ok(out)
}
