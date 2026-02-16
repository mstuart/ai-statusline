use criterion::{criterion_group, criterion_main, Criterion};

fn bench_render(_c: &mut Criterion) {
    // TODO: implement benchmarks
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
