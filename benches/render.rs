use criterion::{Criterion, criterion_group, criterion_main};

fn bench_render(_c: &mut Criterion) {
    // TODO: implement benchmarks
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
