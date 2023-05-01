use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tof_dataclasses::events::RBBinaryDump;
use tof_dataclasses::io::read_file;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::FromRandom;
use tof_dataclasses::events::blob::BlobData;
use std::path::Path;


fn bench_read_file(c: &mut Criterion) {
  c.bench_function("read_file", |b|
                   b.iter(|| read_file(&Path::new("test-data/tof-rb01.robin")).unwrap()));
}

fn bench_rbbinarydump_from_random(c: &mut Criterion) {
  c.bench_function("rbbinarydump_fromrandom", |b|
                   b.iter(|| RBBinaryDump::from_random()));
}

fn bench_rbbinarydump_serialization_circle_helper() {
  let mut data = RBBinaryDump::from_random();
  let stream = data.to_bytestream();
  let data   = RBBinaryDump::from_bytestream(&stream, &mut 0);
}

fn bench_rbbinarydump_serialization_circle(c: &mut Criterion) {
  c.bench_function("rbinbary_dump_serialization_circle", |b|
    b.iter(|| bench_rbbinarydump_serialization_circle_helper()) 
  );
}

fn bench_blobdata_serialization_circle_norandom(c: &mut Criterion) {
  c.bench_function("blobdata_dump_serialization_circle_norandom", |b|
    b.iter(|| {
      let data = BlobData::new();
      let mut data2 = BlobData::new();
      data2.from_bytestream(&data.to_bytestream(), 0, false);
    }) 
  );
}

fn bench_rbbinarydump_serialization_circle_norandom(c: &mut Criterion) {
  c.bench_function("rbinbary_dump_serialization_circle_norandom", |b|
    b.iter(|| {
      let data = RBBinaryDump::new();
      RBBinaryDump::from_bytestream(&data.to_bytestream(), &mut 0);
    }) 
  );
}

criterion_group!(benches,
                 bench_read_file,
                 bench_rbbinarydump_from_random,
                 bench_rbbinarydump_serialization_circle,
                 bench_rbbinarydump_serialization_circle_norandom,
                 bench_blobdata_serialization_circle_norandom);
criterion_main!(benches);
