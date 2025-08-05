//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use rand::Rng;
use rand::distr::StandardUniform;
use rand::prelude::Distribution;

/// Random numbers for testing/benchmarking
pub trait FromRandom {
  fn from_random() -> Self;
}

pub fn rand_vec<T>(size : usize) -> Vec<T> 
  where StandardUniform: Distribution<T> {
  let mut rng = rand::rng();

  let mut random_vector: Vec<T> = Vec::new();
  for _ in 0..size {
    let random_number = rng.random::<T>();
    random_vector.push(random_number);
  }
  return random_vector;
}
