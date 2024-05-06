//! Collections of statistics tools and functions
//!
//! Part of it is copied over from other crates, 
//! since e.g. the statistical crate pulls in 
//! a lot of dependencies, e.g. rand which has 
//! a libc dependency
//!
//!

#[inline(always)]
fn select_pivot<T>(v: &mut [T])
  where T: Copy {
  let idx = rand::random::<usize>() % v.len();
  let tmp = v[0];
  v[0] = v[idx];
  v[idx] = tmp;
}


/// Copy paste from statistical crate
fn partition<T>(v: &mut [T]) -> usize
  where T: PartialOrd + Copy {
  select_pivot(v);
  let pivot = v[0];
  let mut i = 0;
  let mut j = 0;
  let end = v.len() - 1;
  while i < end {
      i += 1;
      if v[i] < pivot {
          v[j] = v[i];
          j += 1;
          v[i] = v[j];
      }

  }
  v[j] = pivot;
  j
}

/// The median is the number separating the higher half of a data sample, a population, or
/// a probability distribution, from the lower half (reference)[http://en.wikipedia.org/wiki/Median)
pub fn median<T>(v: &[T]) -> T
  where T: Copy + Num + NumCast + PartialOrd  {
  assert!(v.len() > 0);
  let mut scratch: Vec<&T> = Vec::with_capacity(v.len());
  scratch.extend(v.iter());
  quicksort(&mut scratch);

  let mid = scratch.len() / 2;
  if scratch.len() % 2 == 1 {
      *scratch[mid]
  } else {
      (*scratch[mid] + *scratch[mid-1]) / num::cast(2).unwrap()
  }
}


/// Copy paste from statistical
pub fn quicksort<T>(v: &mut [T]) 
  where T: PartialOrd + Copy {
  if v.len() <= 1 {
      return
  }
  let pivot = partition(v);
  quicksort(&mut v[..pivot]);
  quicksort(&mut v[(pivot+1)..]);
}

#[test]
fn test_qsort_empty() {
  let mut vec: Vec<f64> = vec![];
  quicksort(&mut vec);
  assert_eq!(vec, vec![]);
}

#[test]
fn test_qsort_small() {
  let len = 10;
  let mut vec = Vec::with_capacity(len);
  for _ in 0..len { vec.push(rand::random::<f64>()); }
  quicksort(&mut vec);
  for i in 0..(len-1) {
      assert!(vec[i] < vec[i+1], "sorted vectors must be monotonically increasing");
  }
}

#[test]
fn test_qsort_large() {
  let len = 1_000_000;
  let mut vec = Vec::with_capacity(len);
  for _ in 0..len { vec.push(rand::random::<f64>()); }
  quicksort(&mut vec);
  for i in 0..(len-1) {
      assert!(vec[i] < vec[i+1], "sorted vectors must be monotonically increasing");
  }
}

#[test]
fn test_qsort_sorted() {
  let len = 1_000;
  let mut vec = Vec::with_capacity(len);
  for n in 0..len { vec.push(n); }
  quicksort(&mut vec);
  for i in 0..(len-1) {
      assert!(vec[i] < vec[i+1], "sorted vectors must be monotonically increasing");
  }
}

#[test]
fn test_qsort_reverse_sorted() {
  let len = 1_000;
  let mut vec = Vec::with_capacity(len);
  for n in 0..len { vec.push(len-n); }
  quicksort(&mut vec);
  for i in 0..(len-1) {
      assert!(vec[i] < vec[i+1], "sorted vectors must be monotonically increasing");
  }
}

#[test]
fn test_median() {
  let vec = vec![1.0, 3.0];
  let diff = abs(median(&vec) - 2.0);

  assert!(diff <= EPSILON);

  let vec = vec![1.0, 3.0, 5.0];
  let diff = abs(median(&vec) - 3.0);

  assert!(diff <= EPSILON);

  let vec = vec![1.0, 3.0, 5.0, 7.0];
  let diff = abs(median(&vec) - 4.0);

  assert!(diff <= EPSILON);
}

