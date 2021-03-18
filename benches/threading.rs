use criterion::{criterion_group, criterion_main, Criterion};
use load::ThreadingLoad;

fn bench_threading(c: &mut Criterion) {}

criterion_group!(benches, bench_threading);
criterion_main!(benches);
