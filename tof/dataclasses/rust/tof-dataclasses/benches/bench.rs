use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tof_dataclasses::events::RBEventMemoryView;
use tof_dataclasses::io::read_file;
use tof_dataclasses::serialization::Serialization;
#[cfg(feature = "random")]
use tof_dataclasses::FromRandom;
//use tof_dataclasses::events::blob::BlobData;
use std::path::Path;


fn bench_read_file(c: &mut Criterion) {
  c.bench_function("read_file", |b|
                   b.iter(|| read_file(&Path::new("test-data/tof-rb01.robin")).unwrap()));
}

cfg_if::cfg_if! {
  if #[cfg(feature = "random")] {
    fn bench_rbmemoryview_from_random(c: &mut Criterion) {
      c.bench_function("rbmemoryview_fromrandom", |b|
                       b.iter(|| RBEventMemoryView::from_random()));
    }

    fn bench_rbmemoryview_serialization_circle_helper() {
      let mut data = RBEventMemoryView::from_random();
      let stream = data.to_bytestream();
      let data   = RBEventMemoryView::from_bytestream(&stream, &mut 0);
    }

    fn bench_rbmemoryview_serialization_circle(c: &mut Criterion) {
      c.bench_function("rbinbary_dump_serialization_circle", |b|
        b.iter(|| bench_rbmemoryview_serialization_circle_helper()) 
      );
    }

    fn bench_rbmemoryview_serialization_circle_norandom(c: &mut Criterion) {
      c.bench_function("rbinbary_dump_serialization_circle_norandom", |b|
        b.iter(|| {
          let data = RBEventMemoryView::new();
          RBEventMemoryView::from_bytestream(&data.to_bytestream(), &mut 0);
        }) 
      );
    }
    
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "random")] {
    criterion_group!(benches,
                    bench_read_file,
                    bench_rbmemoryview_from_random,
                    bench_rbmemoryview_serialization_circle,
                    bench_rbmemoryview_serialization_circle_norandom,
                    //bench_blobdata_serialization_circle_norandom);
                    );
  } else {
    criterion_group!(benches,
                    bench_read_file
                    //bench_blobdata_serialization_circle_norandom);
                    );
  }
}

criterion_main!(benches);
