use core::hint::black_box;
use criterion::{Criterion, criterion_group, criterion_main};

fn roller_benchmark(c: &mut Criterion) {
    c.bench_function("1d20", |b| b.iter(|| dice::roll(black_box("1d20"))));
    c.bench_function("100d20", |b| b.iter(|| dice::roll(black_box("100d20"))));
    let expr = dice::parse(black_box(&("1d20 + ".repeat(50) + "1d20")), false).unwrap();
    c.bench_function("pre_parsed", |b| {
        b.iter(|| dice::roll_expr_with_default_rng(black_box(expr.clone()), dice::Advantage::None))
    });
    c.bench_function("10d20rr<20", |b| b.iter(|| dice::roll(black_box("10d20rr<20"))));
}

criterion_group!(benches, roller_benchmark);
criterion_main!(benches);
