use super::*;
use core::fmt;

#[test]
fn basic() {
    assert_eq!(rformat!("Hello {} {}", "World", "Rust"), "Hello World Rust");
    assert_eq!(rformat!("no placeholders"), "no placeholders");
}

#[test]
fn index_and_reuse() {
    assert_eq!(rformat!("{1} {0} {0}", "a", "b"), "b a a");
    assert_eq!(rformat!("{} {1} {}", "a", "b"), "a b b");
}

#[test]
fn types() {
    assert_eq!(rformat!("{:?}", "s"), "\"s\"");
    assert_eq!(
        rformat!("{:#x} {:X} {:#o} {:#b}", 255, 255, 8, 5),
        "0xff FF 0o10 0b101"
    );
    assert_eq!(rformat!("{:e} {:E}", 12345.678, 12345.678), "1.2345678e4 1.2345678E4");
    assert_eq!(rformat!("{:#x}", -1), "0xffffffff");
}

#[derive(Debug, FormatArg)]
struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[test]
fn custom_arg_and_pretty_debug() {
    assert_eq!(rformat!("{}", Point { x: 1, y: 2 }), "(1, 2)");
    assert_eq!(rformat!("{:?}", Point { x: 1, y: 2 }), "Point { x: 1, y: 2 }");
    let out = rformat!("{:#?}", Point { x: 1, y: 2 });
    assert!(out.contains('\n'));
    assert_eq!(
        "{:x}".try_format(&(Point { x: 1, y: 2 },)),
        Err(FormatError::UnsupportedFormatType)
    );
}

#[derive(FormatArg)]
struct DisplayOnly;

impl fmt::Display for DisplayOnly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("display-only")
    }
}

#[test]
fn derive_display_only() {
    assert_eq!(rformat!("{}", DisplayOnly), "display-only");
    assert_eq!(
        "{:?}".try_format(&(DisplayOnly,)),
        Err(FormatError::UnsupportedFormatType)
    );
}

#[derive(Debug, FormatArg)]
struct DebugOnly;

#[test]
fn derive_debug_only() {
    assert_eq!(rformat!("{:?}", DebugOnly), "DebugOnly");
    assert_eq!(rformat!("{}", DebugOnly), "DebugOnly");
}

#[derive(Debug, FormatArg)]
struct Wrapper<T>(T);

impl<T: fmt::Display> fmt::Display for Wrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

#[test]
fn derive_generic() {
    assert_eq!(rformat!("{}", Wrapper(7)), "<7>");
    assert_eq!(rformat!("{:?}", Wrapper("s")), "Wrapper(\"s\")");
}

#[test]
fn align_fill() {
    assert_eq!(
        rformat!("{:<5}|{:>5}|{:^5}", "a", "a", "a"),
        "a    |    a|  a  "
    );
    assert_eq!(rformat!("{:_^7}", "ab"), "__ab___");
    assert_eq!(rformat!("{:5}", 42), "   42");
}

#[test]
fn sign_and_zero() {
    assert_eq!(rformat!("{:+} {:+}", 42, -7), "+42 -7");
    assert_eq!(rformat!("{:05}", 42), "00042");
    assert_eq!(rformat!("{:+05}", 42), "+0042");
    assert_eq!(rformat!("{:#010x}", 255), "0x000000ff");
    assert_eq!(rformat!("{:05x}", 255), "000ff");
    assert_eq!(rformat!("{:08.4}", 42), "00000042");
    // the `0` flag overrides fill/align for numerics, like std
    assert_eq!(rformat!("{:<05}", 42), "00042");
    assert_eq!(rformat!("{:_<05}", 42), "00042");
    assert_eq!(rformat!("{:<06}", -42), "-00042");
    // ...but is ignored for non-numerics, like std
    assert_eq!(rformat!("{:05}", "ab"), "ab   ");
    assert_eq!(rformat!("{:05}", true), "true ");
    // std never attaches `+` to NaN
    assert_eq!(rformat!("{:+}", f64::NAN), "NaN");
    assert_eq!(rformat!("{:+05}", f64::NAN), "00NaN");
}

#[test]
fn precision() {
    assert_eq!(rformat!("{:.2}", 1.234567), "1.23");
    assert_eq!(rformat!("{:.2e}", 1.234567), "1.23e0");
    assert_eq!(rformat!("{:.3}", "abcdef"), "abc");
    assert_eq!(rformat!("{:.5}", 42), "42"); // precision is ignored for integers, like std
}

#[test]
fn dynamic_width_precision() {
    assert_eq!(rformat!("{:1$}", "ab", 5), "ab   ");
    assert_eq!(rformat!("{0:>1$}", "ab", 5), "   ab");
    assert_eq!(rformat!("{:.1$}", 1.234567, 2), "1.23");
    assert_eq!(rformat!("{1:.0$}", 2, 1.234567), "1.23");
}

#[test]
fn builder() {
    assert_eq!(
        "{} + {} = {2}".builder().arg(&1).arg(&2).arg(&3).build(),
        "1 + 2 = 3"
    );
    assert_eq!("Hi {}".builder().arg("Rust").build(), "Hi Rust");
    let width = 5;
    assert_eq!("{:1$}".builder().arg("ab").arg(&width).build(), "ab   ");
    assert_eq!("{0} {0}".builder().arg(&7).build(), "7 7");
    assert_eq!(
        "{:x}".builder().arg("str").try_build(),
        Err(FormatError::UnsupportedFormatType)
    );
    assert!("{}".builder().try_build().is_err());
}

#[test]
fn escape() {
    assert_eq!(rformat!("{{}} {{{}}}", 1), "{} {1}");
    assert_eq!(rformat!("}}"), "}");
}

#[test]
fn errors() {
    assert_eq!(
        "{} {}".try_format(&("only",)),
        Err(FormatError::InsufficientParameters)
    );
    assert_eq!("{".try_format(&()), Err(FormatError::InvalidFormatString));
    assert_eq!("}".try_format(&()), Err(FormatError::InvalidFormatString));
    assert_eq!("{name}".try_format(&("v",)), Err(FormatError::InvalidFormatString));
    assert_eq!("{:0$}".try_format(&(42, 5)), Err(FormatError::InvalidFormatString));
    assert_eq!("{:x}".try_format(&("str",)), Err(FormatError::UnsupportedFormatType));
    assert_eq!("{:p}".try_format(&(1,)), Err(FormatError::UnsupportedFormatType));
    assert_eq!("{:1$}".try_format(&("ab", "cd")), Err(FormatError::ExpectedUsize));
}
