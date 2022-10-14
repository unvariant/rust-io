use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_io::*;

pub fn simd_benchmark(c: &mut Criterion) {
    c.bench_function("i32 atoi", |b| b.iter(|| unsafe { i32_from_str16_sse(black_box(b"-000000000000000000001234567890")) }));
}

pub fn std_benchmark(c: &mut Criterion) {
    c.bench_function("i32 parse", |b| b.iter(|| black_box(b"-000000000000000000001234567890".parse::<i32>().unwrap())));
}

criterion_group!(benches, simd_benchmark, std_benchmark);
criterion_main!(benches);