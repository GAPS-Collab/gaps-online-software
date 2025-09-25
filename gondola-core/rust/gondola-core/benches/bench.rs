use criterion::{
    criterion_group,
    criterion_main,
    Criterion
};

use rand::Rng;
use gondola_core::random::rand_vec;

//----------------------------------------

fn bench_rand_vec(c: &mut Criterion) {
  fn wrap_rand_vec() {
    let size : usize = 50000;
    let data = rand_vec::<f32>(size);
  }
  c.bench_function("rand_vec", |b|
                   b.iter(|| wrap_rand_vec()));
}

//----------------------------------------

fn bench_roll(c: &mut Criterion) {
  let size : usize = 1024;
  let mut data = rand_vec::<f32>(size);
  use gondola_core::calibration::tof::roll;
  c.bench_function("roll", |b|
                    b.iter(|| roll::<f32>(&mut data, 512)));
}

//----------------------------------------

fn bench_parse_u64_generic(c: &mut Criterion) {
  let size : usize = 10000;
  let data = rand_vec::<u8>(size);
  use gondola_core::io::parsers::parse_u64_new;
  c.bench_function("parse_u64_generic", |b|
                    b.iter(|| parse_u64_new(&data,&mut 5000)));
}

//----------------------------------------

fn bench_parse_u64(c: &mut Criterion) {
  let size : usize = 10000;
  let data = rand_vec::<u8>(size);
  use gondola_core::io::parsers::parse_u64;
  c.bench_function("parse_u64", |b|
                    b.iter(|| parse_u64(&data, &mut 5000)));
}

//---------------------------------------

fn bench_get_max_value_idx(c: &mut Criterion) {
  let size : usize = 50000;
  let data = rand_vec::<f32>(size);
  fn wrap_get_max_value_idx(data : &Vec<f32>) {
    use gondola_core::tof::algorithms::get_max_value_idx;
    let size = data.len();
    for _ in 0..10000 {
      let start_idx : usize = rand::rng().random_range(0..size); 
      let n_idx = size - start_idx;
      get_max_value_idx(&data, start_idx, n_idx);
    }
  }
  c.bench_function("get_max_value_idx", |b|
                   b.iter(|| wrap_get_max_value_idx(&data)));
}


//---------------------------------------

fn bench_clean_spikes(c: &mut Criterion) {
  let size : usize = 1024;
  let mut data = rand_vec::<f32>(size);
  use gondola_core::calibration::tof::clean_spikes;
  c.bench_function("clean_spikes", |b|
                    b.iter(|| clean_spikes(&mut data, true)));
}
  
//---------------------------------------

//fn bench_search_for_u16(c: &mut Criterion) {
//  let size : usize = 10000;
//  let mut data = rand_vec::<u8>(size);
//  // make sure there is no 170 in there
//  for k in 0..data.len() {
//    if data[k] == 170 { 
//      data[k] = 0;
//    }
//  }
//  // insert the 170,179 in a defined place
//  //let idx = rand::rng.random_range(0..data.len() - 2);
//  let idx = data.len() - 2;
//  data[idx] = 170;
//  data[idx + 1] = 170;
//
//  use gondola_core::io::serialization::search_for_u16;
//  c.bench_function("search_for_u16", |b|
//                    b.iter(|| search_for_u16(0xAA, &data, 0)));
//}
  
//---------------------------------------

fn bench_seek_marker_arr(c: &mut Criterion) {
  let size : usize = 10000;
  let mut data = rand_vec::<u8>(size);
  // make sure there is no 170 in there
  for k in 0..data.len() {
    if data[k] == 170 { 
      data[k] = 0;
    }
  }
  // insert the 170,179 in a defined place
  //let idx = rand::rng.random_range(0..data.len() - 2);
  let idx = data.len() - 2;
  data[idx] = 170;
  data[idx + 1] = 170;

  use gondola_core::io::serialization::seek_marker;
  c.bench_function("seek_marker_arr", |b|
                    b.iter(|| seek_marker(&data.as_slice(), 0xAAAA, 0)));
}
  
//---------------------------------------

fn bench_seek_marker_vec(c: &mut Criterion) {
  let size : usize = 10000;
  let mut data = rand_vec::<u8>(size);
  // make sure there is no 170 in there
  for k in 0..data.len() {
    if data[k] == 170 { 
      data[k] = 0;
    }
  }
  // insert the 170,179 in a defined place
  //let idx = rand::rng.random_range(0..data.len() - 2);
  let idx = data.len() - 2;
  data[idx] = 170;
  data[idx + 1] = 170;

  use gondola_core::io::serialization::seek_marker;
  c.bench_function("seek_marker_vec", |b|
                    b.iter(|| seek_marker(&data, 0xAAAA, 0)));
}
  
//---------------------------------------



//fn bench_rb_waveform_adc_py(c: &mut Criterion) {
//  use gondola_core::events::RBWaveform;
//  use gondola_core::random::FromRandom;
//  let wf = RBWaveform::from_random();
//  c.bench_function("rb_waveform_adc_py", |b|
//                    b.iter(|| wf.adc_a_py()));
//}

//---------------------------------------

//criterion_group! {
//  name = benches;
//  config = Criterion::default().sample_size(500); // Set default sample size to 500
//  targets = bench_my_function
//}

//---------------------------------------

criterion_group!(benches,
                 bench_rand_vec,
                 bench_roll,
                 bench_get_max_value_idx,
                 bench_clean_spikes,
                 bench_parse_u64,
                 bench_parse_u64_generic,
                 bench_seek_marker_arr,
                 bench_seek_marker_vec);
criterion_main!(benches);

