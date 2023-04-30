use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tof_dataclasses::events::RBBinaryDump;
use tof_dataclasses::io::read_file;
use std::path::Path;


fn bench_read_file(c: &mut Criterion) {
  c.bench_function("read_file", |b|
                   b.iter(|| read_file(&Path::new("test-data/tof-rb01.robin")).unwrap()));
}

fn bench_reverse(c: &mut Criterion) {
  let mut vec = vec![1, 2, 3, 4, 5];
  c.bench_function("reverse", |b| b.iter(|| vec.reverse()));
}

criterion_group!(benches, bench_read_file);
criterion_main!(benches);
