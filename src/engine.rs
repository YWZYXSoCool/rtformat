use alloc::string::String;
use core::fmt;
use core::fmt::Write as _;

use crate::arg::{FormatArg, FormatType};
use crate::error::FormatError;
use crate::spec::{Align, Spec};

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

/// The `#` radix prefix for each integer format type. std always uses a
/// lowercase `0x` prefix, even for `{:X}` (only the digits are uppercased).
fn radix_prefix(ty: FormatType) -> &'static str {
    match ty {
        FormatType::LowerHex | FormatType::UpperHex => "0x",
        FormatType::Octal => "0o",
        FormatType::Binary => "0b",
        _ => "",
    }
}

/// Maps a write rejected by the output sink to [`FormatError::WriteFailed`].
fn note_write(result: fmt::Result) -> Result<(), FormatError> {
    result.map_err(|_| FormatError::WriteFailed)
}

/// Writes `fill` repeated `n` times to the sink, without allocating.
///
/// Copies of `fill` are packed into a stack buffer and written in chunks, so
/// a wide pad costs a few `write_str` calls rather than one per character.
fn write_fill(w: &mut dyn fmt::Write, fill: char, n: usize) -> Result<(), FormatError> {
    const CAP: usize = 256;
    let mut buf = [0u8; CAP];
    let mut char_buf = [0u8; 4];
    let piece = fill.encode_utf8(&mut char_buf).as_bytes();
    let plen = piece.len();
    let per_chunk = CAP / plen;

    let mut remaining = n;

    // Full chunks: only when there are enough fills to need them, lay down
    // `per_chunk` copies once and reuse that buffer for every chunk write.
    if remaining >= per_chunk {
        for i in 0..per_chunk {
            buf[i * plen..(i + 1) * plen].copy_from_slice(piece);
        }
        let full = core::str::from_utf8(&buf[..per_chunk * plen]).expect("whole copies of a char");
        while remaining >= per_chunk {
            note_write(w.write_str(full))?;
            remaining -= per_chunk;
        }
    }

    // Tail (or the whole thing, when `n < per_chunk`): fill only the copies
    // still needed, so small pads cost O(n) rather than O(CAP) setup.
    if remaining > 0 {
        for i in 0..remaining {
            buf[i * plen..(i + 1) * plen].copy_from_slice(piece);
        }
        let rest = core::str::from_utf8(&buf[..remaining * plen]).expect("whole copies of a char");
        note_write(w.write_str(rest))?;
    }
    Ok(())
}

/// Writes `body` padded to `spec.width`; when no alignment is given,
/// `default` is used (right for numerics, left otherwise — same as std).
fn write_padded(
    w: &mut dyn fmt::Write,
    spec: Spec,
    default: Align,
    total_len: usize,
    body: impl FnOnce(&mut dyn fmt::Write) -> Result<(), FormatError>,
) -> Result<(), FormatError> {
    let Some(width) = spec.width else {
        return body(w);
    };
    if total_len >= width {
        return body(w);
    }
    let pad = width - total_len;
    match spec.align.unwrap_or(default) {
        Align::Left => {
            body(w)?;
            write_fill(w, spec.fill, pad)
        }
        Align::Right => {
            write_fill(w, spec.fill, pad)?;
            body(w)
        }
        Align::Center => {
            let left = pad / 2;
            write_fill(w, spec.fill, left)?;
            body(w)?;
            write_fill(w, spec.fill, pad - left)
        }
    }
}

/// Truncates `s` to at most `max` characters (char count, same as std).
fn truncate_chars(s: &mut String, max: usize) {
    if s.chars().count() > max
        && let Some((end, _)) = s.char_indices().nth(max)
    {
        s.truncate(end);
    }
}

/// Renders one argument according to `spec` into `w`, reusing `scratch` as
/// the buffer for the bare value.
///
/// # Errors
///
/// Returns [`FormatError::UnsupportedFormatType`] if the argument does not
/// support the requested format type, and [`FormatError::WriteFailed`] if
/// the sink rejects a write.
pub(crate) fn render_arg(
    arg: &dyn FormatArg,
    spec: Spec,
    w: &mut dyn fmt::Write,
    scratch: &mut String,
) -> Result<(), FormatError> {
    scratch.clear();
    write!(scratch, "{}", Core { arg, spec }).map_err(|_| FormatError::UnsupportedFormatType)?;

    if !arg.is_numeric() {
        // The `0` flag has no effect on non-numerics (same as std): only the
        // spec's own fill/align participate in padding.
        if let Some(p) = spec.precision {
            truncate_chars(scratch, p);
        }
        // Without a width there is nothing to pad, so skip the char count.
        if spec.width.is_none() {
            return note_write(w.write_str(scratch));
        }
        let total = scratch.chars().count();
        return write_padded(w, spec, Align::Left, total, |w| {
            note_write(w.write_str(scratch))
        });
    }

    let (sign, digits) = if let Some(rest) = scratch.strip_prefix('-') {
        ("-", rest)
    } else if spec.sign_plus && scratch != "NaN" {
        // std never attaches `+` to NaN; the float impls render it as
        // exactly "NaN".
        ("+", scratch.as_str())
    } else {
        ("", scratch.as_str())
    };
    let prefix = if spec.alternate {
        radix_prefix(spec.ty)
    } else {
        ""
    };

    // std semantics: when the `0` flag is set, numerics are always
    // sign-aware zero-filled — overriding any fill/align. Precision is
    // ignored for integers (same as std).
    if spec.zero {
        note_write(w.write_str(sign))?;
        note_write(w.write_str(prefix))?;
        if let Some(width) = spec.width {
            let cur = sign.len() + prefix.len() + digits.len();
            if cur < width {
                write_fill(w, '0', width - cur)?;
            }
        }
        return note_write(w.write_str(digits));
    }

    let total = sign.len() + prefix.len() + digits.len();
    write_padded(w, spec, Align::Right, total, |w| {
        note_write(w.write_str(sign))?;
        note_write(w.write_str(prefix))?;
        note_write(w.write_str(digits))
    })
}
