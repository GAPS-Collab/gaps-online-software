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


//---------------------------------------

fn bench_get_max_value_idx(c: &mut Criterion) {
  let size : usize = 50000;
  let data = rand_vec::<f32>(size);
  fn wrap_get_max_value_idx(data : &Vec<f32>) {
    use gondola_core::tof::algorithms::get_max_value_idx;
    let size = data.len();
    for _ in 0..10000 {
      let start_idx : usize = rand::thread_rng().gen_range(0..size); 
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

fn bench_rb_waveform_adc_py(c: &mut Criterion) {
  use gondola_core::events::RBWaveform;
  use gondola_core::random::FromRandom;
  let wf = RBWaveform::from_random();
  c.bench_function("rb_waveform_adc_py", |b|
                    b.iter(|| wf.adc_a_py()));
}
//---------------------------------------

criterion_group!(benches,
                 bench_rand_vec,
                 bench_roll,
                 bench_get_max_value_idx,
                 bench_clean_spikes);
criterion_main!(benches);

