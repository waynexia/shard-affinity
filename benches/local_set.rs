use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use load::LocalSetLoad;
use rand::random;

fn prepare_data() -> LocalSetLoad {
    todo!()
}

fn bench_local_set(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("read");
    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // b.iter(|| iter::repeat(0u8).take(size).collect::<Vec<_>>());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_local_set);
criterion_main!(benches);
