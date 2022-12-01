/***********************************/

use cargo::constants::NWORDS;

/***********************************/

pub fn find_interpolated_time(adc       : [f64;NWORDS],
                              times     : [f64;NWORDS], 
                              threshold : f32,
                              idx       : i32,
                              size      : usize ) -> f64 
{
  let mut time :f64;
  threshold = threshold.abs();
  let lval  = (adc[idx]).abs();
  let mut hval : f64; 
  if size == 1 {
    hval = (adc[idx+1]).abs();
  } else {
  for n in idx+1..idx+size {
    hval = adc[n].abs();
    if (hval>=threshold) && (threshold<=lval) { // Threshold crossing?
      idx = n-1; // Reset idx to point before crossing
      break;
      }
    lval = hval;
    }
  if ( lval > threshold) && (size != 1) {
    return times[idx];
  } else if (lval == hval) {
    return times[idx];
  } else {
    time = adc[idx] + (threshold-lval)/(hval-lval) * (adc[idx+1]-adc[idx]);
    //float time = WaveTime[idx] +  
    //  (thresh-lval)/(hval-lval) * (WaveTime[idx+1]-WaveTime[idx]) ;
    return time;
    }
}



