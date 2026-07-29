# rtformat

[![Crates.io](https://img.shields.io/crates/v/rtformat.svg)](https://crates.io/crates/rtformat)
[![Docs.rs](https://docs.rs/rtformat/badge.svg)](https://docs.rs/rtformat)
[![License](https://img.shields.io/crates/l/rtformat.svg)](#license)

Runtime string formatting for Rust, with `std::fmt`-compatible placeholder syntax.

Rust's built-in `format!` macro requires the format string to be a compile-time
literal. `rtformat` lets you take a template string **at runtime** — from a config
file, a database, or user input — and format it with the same placeholder syntax
you already know.

## Features

- `{}` implicit positional arguments; `{0}` / `{1}` explicit indices (reusable)
- Types: `{:?}` `{:#?}` `{:x}` `{:X}` `{:o}` `{:b}` `{:e}` `{:E}`
- Alignment and fill: `{:<10}` `{:>10}` `{:^10}` `{:_>10}`
- Sign / radix prefix / zero padding: `{:+}` `{:#x}` `{:010}`
- Width and precision: `{:.3}` `{:1$}` `{:.1$}` (`n$` takes the width/precision
  from argument `n`)
- `{{` / `}}` escapes
- Custom argument types via `#[derive(FormatArg)]`
- Fallible formatting (`try_format`) and an incremental builder API

Not supported: named arguments `{name}`, pointers `{:p}`, hexadecimal debug
`{x?}` / `{X?}`.

## Usage

Add the dependency:

```toml
[dependencies]
rtformat = "0.1"
```

### The `rformat!` macro

```rust
use rtformat::rformat;

assert_eq!(rformat!("Hello {}!", "world"), "Hello world!");
assert_eq!(rformat!("{0} + {1} = {2}", 1, 2, 3), "1 + 2 = 3");
assert_eq!(rformat!("{:#010x}", 255), "0x000000ff");
assert_eq!(rformat!("{:.2}", 3.14159), "3.14");
```

### The `Format` trait

`Format` is implemented for `str` (and available on `String`, `Box<str>`, etc.
through deref):

```rust
use rtformat::Format;

assert_eq!("{} + {} = {2}".format(&(1, 2, 3)), "1 + 2 = 3");

// Fallible variants, for templates that may be invalid:
let result = "{}".try_format(&("hello",));
assert_eq!(result.as_deref(), Ok("hello"));
```

### Builder API

Add arguments one at a time:

```rust
use rtformat::Format;

let s = "{} + {} = {2}".builder().arg(&1).arg(&2).arg(&3).build();
assert_eq!(s, "1 + 2 = 3");
```

### Custom argument types

Types implementing `Display` and/or `Debug` can derive `FormatArg`. The derive
adapts: `{}` prefers `Display` and falls back to `Debug`, while `{:?}` / `{:#?}`
require `Debug`:

```rust
use core::fmt;
use rtformat::{rformat, Format, FormatArg};

#[derive(Debug, FormatArg)]
struct Color(u8, u8, u8);

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

assert_eq!(rformat!("{}", Color(255, 128, 0)), "#ff8000");
```

For full control over each format type, implement `FormatArg` manually instead.

### Prelude

All commonly used items are available through the prelude:

```rust
use rtformat::prelude::*;

assert_eq!(rformat!("Hello {}!", "world"), "Hello world!");
```

## Workspace layout

| Crate | Description |
| ----- | ----------- |
| [`rtformat`](.) | The formatting engine, traits, and the `rformat!` macro |
| [`rtformat-derive`](rtformat-derive) | The `#[derive(FormatArg)]` proc-macro |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
