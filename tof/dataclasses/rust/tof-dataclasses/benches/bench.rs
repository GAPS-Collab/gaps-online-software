use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tof_dataclasses::events::RBBinaryDump;


fn bench_reverse(c: &mut Criterion) {
        let mut vec = vec![1, 2, 3, 4, 5];
            c.bench_function("reverse", |b| b.iter(|| vec.reverse()));
}

criterion_group!(benches, bench_reverse);
criterion_main!(benches);
