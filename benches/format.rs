//! Criterion benchmarks for `rtformat`.
//!
//! They are organized around the two cost regimes of the crate:
//!
//! - **One-shot** (`rformat!` / `Format::format`): the template is parsed on
//!   every call, so these are dominated by parse cost.
//! - **Template reuse** (`Template::parse` once, format many): parsing is
//!   amortized away, so these are dominated by per-argument rendering.
//!
//! Run with `cargo bench`. Numbers are not committed; Criterion keeps the
//! previous run under `target/criterion/` and reports regressions/improvements
//! against it automatically.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use rtformat::{Format, Template, rformat};

/// One-shot APIs: each iteration re-parses the template.
fn one_shot(c: &mut Criterion) {
    let mut group = c.benchmark_group("one_shot");

    group.bench_function("simple_display", |b| {
        b.iter(|| rformat!("Hello {}!", black_box("world")))
    });

    group.bench_function("three_args_reuse", |b| {
        b.iter(|| black_box("{} + {} = {2}").format(black_box(&(1, 2, 3))))
    });

    group.bench_function("complex_specs", |b| {
        b.iter(|| {
            black_box("{:#010x} | {:_>12} | {:.3} | {:+}").format(black_box(&(
                255u32,
                "pad",
                123.4567f64,
                42i32,
            )))
        })
    });

    group.finish();
}

/// Reused templates: parsing happens once, outside the measured loop.
fn template_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_reuse");

    let simple = Template::parse("{} + {} = {2}").unwrap();
    group.bench_function("simple", |b| {
        b.iter(|| simple.format(black_box(&(1, 2, 3))))
    });

    let specs = Template::parse("{:#010x} | {:_>12} | {:.3} | {:+}").unwrap();
    group.bench_function("complex_specs", |b| {
        b.iter(|| specs.format(black_box(&(255u32, "pad", 123.4567f64, 42i32))))
    });

    // A single very long value: stresses the bare-value write and the
    // char-counting in the non-numeric render path.
    let long = "a".repeat(10_000);
    let passthrough = Template::parse("{}").unwrap();
    group.bench_function("long_string", |b| {
        b.iter(|| passthrough.format(black_box(&(long.as_str(),))))
    });

    // Wide padding: stresses `write_fill` (one sink call per fill char).
    let wide = Template::parse("{:_>256}").unwrap();
    group.bench_function("wide_padding", |b| {
        b.iter(|| wide.format(black_box(&("x",))))
    });

    group.finish();
}

/// Argument-count scaling on the reuse path, to expose per-argument overhead.
fn arg_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("arg_scaling");

    let t1 = Template::parse("{}").unwrap();
    group.bench_function("n1", |b| b.iter(|| t1.format(black_box(&("a",)))));

    let t4 = Template::parse("{} {} {} {}").unwrap();
    group.bench_function("n4", |b| {
        b.iter(|| t4.format(black_box(&("a", "b", "c", "d"))))
    });

    let t16_src = vec!["{}"; 16].join(" ");
    let t16 = Template::parse(&t16_src).unwrap();
    group.bench_function("n16", |b| {
        b.iter(|| {
            t16.format(black_box(&(
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
            )))
        })
    });

    group.finish();
}

/// Builder path with a batch of same-typed arguments.
fn builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    let nums = [1, 2, 3, 4, 5, 6, 7, 8];
    group.bench_function("eight_ints", |b| {
        b.iter(|| {
            black_box("{} {} {} {} {} {} {} {}")
                .builder()
                .args(black_box(nums.iter()))
                .build()
        })
    });

    group.finish();
}

fn config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
}

criterion_group! {
    name = benches;
    config = config();
    targets = one_shot, template_reuse, arg_scaling, builder
}
criterion_main!(benches);
