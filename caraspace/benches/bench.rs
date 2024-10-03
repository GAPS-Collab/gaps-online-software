use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion
};
use rand::Rng;
use std::collections::VecDeque;

use caraspace::prelude::*;

fn bench_parse_u32(c : &mut Criterion) {
  let mut rng = rand::thread_rng();

  // Generate a random Vec<u8>
  let data: Vec<u8> = (0..100).map(|_| rng.gen()).collect(); // Change 100 to any size you want

  c.bench_function("parse_u32 generic from Vec<u8>", |b| {
      b.iter(|| parse_u32new(black_box(&data), &mut 0))
  });
  
  c.bench_function("parse_u32 from Vec<u8>", |b| {
      b.iter(|| parse_u32(black_box(&data), &mut 0))
  });

  // Generate a random VecDeque<u8>
  let deque_data: VecDeque<u8> = data.clone().into_iter().collect();
  let array_data = data.as_slice();
  c.bench_function("parse_u32 from array", |b| {
      b.iter(|| parse_u32new(black_box(&array_data), &mut 0))
  });
}

// Group benchmarks
criterion_group!(benches, bench_parse_u32);
criterion_main!(benches);
