//! Differential tests against `std::format!`: every case asserts that
//! `rformat!` produces exactly the same output as std for the same
//! template and arguments.

use crate::Format as _;

macro_rules! assert_same {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        assert_eq!(
            $crate::rformat!($fmt $(, $arg)*),
            ::std::format!($fmt $(, $arg)*),
            "output differs from std for {:?}",
            $fmt,
        );
    };
}

#[test]
fn basic() {
    assert_same!("Hello {} {}", "World", "Rust");
    assert_same!("no placeholders");
    assert_same!("{{}} {{{}}}", 1);
    assert_same!("}}");
    assert_same!("{1} {0} {0}", "a", "b");
    assert_same!("{} {1} {}", "a", "b");
    assert_same!("{0} {0}", 7);
    assert_same!("{0:5} | {0:.2}", 1.234567);
}

#[test]
fn integers() {
    for v in [0i32, 1, 42, -7, -1, 255, i32::MIN, i32::MAX] {
        assert_same!("{}", v);
        assert_same!("{:?}", v);
        assert_same!("{:#?}", v);
        assert_same!("{:x}", v);
        assert_same!("{:X}", v);
        assert_same!("{:o}", v);
        assert_same!("{:b}", v);
        assert_same!("{:#x}", v);
        assert_same!("{:#X}", v);
        assert_same!("{:#o}", v);
        assert_same!("{:#b}", v);
    }
    assert_same!("{}", u128::MAX);
    assert_same!("{:#x}", i128::MIN);
    assert_same!("{}", usize::MAX);
    assert_same!("{}", i8::MIN);
    assert_same!("{:b}", -1i8);
    assert_same!("{:#040x}", u128::MAX);
}

#[test]
fn floats() {
    for v in [
        0.0f64, -0.0, 3.5, -2.5, 1.234567, 12345.678, -12345.678, 1e100, -1e-100,
    ] {
        assert_same!("{}", v);
        assert_same!("{:?}", v);
        assert_same!("{:e}", v);
        assert_same!("{:E}", v);
        assert_same!("{:+}", v);
        assert_same!("{:.2}", v);
        assert_same!("{:.0}", v);
        assert_same!("{:.5e}", v);
        assert_same!("{:+.2}", v);
    }
    assert_same!("{}", f64::INFINITY);
    assert_same!("{}", f64::NEG_INFINITY);
    assert_same!("{}", f64::NAN);
    assert_same!("{}", f32::NAN);
    assert_same!("{:e}", f64::NAN);
    assert_same!("{:+}", f64::NAN);
    assert_same!("{:+05}", f64::NAN);
    assert_same!("{:05}", f64::NAN);
    assert_same!("{:+}", f64::INFINITY);
    assert_same!("{:+e}", f64::INFINITY);
    assert_same!("{:05}", f64::INFINITY);
    assert_same!("{:05}", f64::NEG_INFINITY);
    assert_same!("{:5}", f64::INFINITY);
    assert_same!("{:+}", -0.0f64);
    assert_same!("{:05}", -0.0f64);
    assert_same!("{:.2}", -0.0f64);
}

#[test]
fn text() {
    for v in ["", "a", "ab", "hello world", "héllo", "你好"] {
        assert_same!("{}", v);
        assert_same!("{:?}", v);
        assert_same!("{:5}", v);
        assert_same!("{:<5}", v);
        assert_same!("{:>5}", v);
        assert_same!("{:^6}", v);
        assert_same!("{:_^7}", v);
        assert_same!("{:.3}", v);
        assert_same!("{:8.3}", v);
        // the `0` flag has no effect on non-numerics
        assert_same!("{:05}", v);
        assert_same!("{:<05}", v);
    }
    assert_same!("{}", 'a');
    assert_same!("{:?}", 'a');
    assert_same!("{:5}", 'a');
    assert_same!("{:04}", 'a');
    assert_same!("{}", true);
    assert_same!("{:?}", true);
    assert_same!("{:7}", true);
    assert_same!("{:07}", true);
    assert_same!("{:#?}", "s");
}

#[test]
fn width_align_sign_zero() {
    for v in [0i32, 42, -7] {
        assert_same!("{:5}", v);
        assert_same!("{:<5}", v);
        assert_same!("{:>5}", v);
        assert_same!("{:^6}", v);
        assert_same!("{:*<6}", v);
        assert_same!("{:_>6}", v);
        assert_same!("{:05}", v);
        // std: the `0` flag overrides fill/align for numerics
        assert_same!("{:<05}", v);
        assert_same!("{:^07}", v);
        assert_same!("{:_<05}", v);
        assert_same!("{:+}", v);
        assert_same!("{:+5}", v);
        assert_same!("{:+05}", v);
        assert_same!("{:-05}", v);
        assert_same!("{:03}", v);
    }
    for v in [3.5f64, -2.5, 1.234567] {
        assert_same!("{:8}", v);
        assert_same!("{:<8}", v);
        assert_same!("{:08.2}", v);
        assert_same!("{:07.2}", v);
        assert_same!("{:+08.2}", v);
        assert_same!("{:012.2e}", v);
    }
    assert_same!("{:#010x}", 255);
    assert_same!("{:05x}", 255);
    assert_same!("{:03}", -5);
    assert_same!("{:<06}", -42);
    assert_same!("{:^07}", -42);
}

#[test]
fn precision() {
    assert_same!("{:.2}", 1.234567);
    assert_same!("{:.2e}", 1.234567);
    assert_same!("{:.3}", "abcdef");
    assert_same!("{:.5}", 42);
    assert_same!("{:.0}", 42);
    assert_same!("{:.0}", 3.7);
    assert_same!("{:.10}", -42);
    assert_same!("{:8.4}", 42);
    // precision is ignored for integers; width and the `0` flag still apply
    assert_same!("{:08.4}", 42);
    assert_same!("{:05.3}", 42);
    assert_same!("{:<08.4}", 42);
    assert_same!("{:08.4}", -42);
    assert_same!("{:8.4}", -42);
    assert_same!("{:10.5}", 42);
}

#[test]
fn dynamic_width_precision() {
    assert_same!("{:1$}", "ab", 5);
    assert_same!("{0:>1$}", "ab", 5);
    assert_same!("{:.1$}", 1.234567, 2);
    assert_same!("{1:.0$}", 2, 1.234567);
    assert_same!("{:01$}", 42, 6);
    assert_same!("{:+01$}", 42, 6);
    assert_same!("{2:01$} {0}", "x", 8, 42);
}
