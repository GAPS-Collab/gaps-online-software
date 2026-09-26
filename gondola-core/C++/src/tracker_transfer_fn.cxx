#ifdef BUILD_CXX_DB
#include <format>
#include <iostream>

#include "sqlite_orm.h"
#include "spdlog/spdlog.h"
#include "database/tracker_strip.hpp"
#include "database/tracker_transfer_fn.hpp"

using namespace sqlite_orm;
using namespace result;

namespace g = gondola;

auto g::TrackerStripTransferFunction::to_string() const -> std::string {
  auto repr = std::format("<TrackerStripTransferFunction [{}]:", strip_id);
  return repr;
  repr += std::format("\n   vid           : {}", volume_id);
  repr += "\n   UTC Timestamps (Begin/End):";
  repr += std::format("\n   {}/{}", utc_timestamp_start, utc_timestamp_stop);    
  if (name != "") {
    repr += format("\n   name     : {}", name); 
  }
  repr += std::format("\n  Poly A {}*adc + {}*adc + {}*(adc**2) for adc < 190", pol_a2_0, pol_a2_1, pol_a2_2);
  repr += std::format("\n  Poly B    :{}*adc + {}*adc + {}*(adc**2) + {}*(adc**3) for 190 < adc <= 500", pol_b3_0, pol_b3_1, pol_b3_2, pol_b3_3);
  repr += std::format("\n  Poly C    :{}*adc + {}*adc + {}*(adc**2) + {}*(adc**3) for 500 < adc <= 900", pol_c3_0, pol_c3_1, pol_c3_2, pol_c3_3);
  repr += std::format("\n  Poly D    :{}*adc + {}*adc + {}*(adc**2) + {}*(adc**3) for 900 < adc <= 1600>", pol_d3_0, pol_d3_1, pol_d3_2, pol_d3_3);
  return repr;
}
  
auto g::TrackerStripTransferFunction::evaluate(f32 adc) const -> f32 {
  if (adc < 0.0) {
    return 0.0;
  }
  if (adc <= 190.0) {
    return pol_a2_0 + pol_a2_1*adc + pol_a2_2*(pow(adc,2));
  }
  if ((190.0 < adc) && (adc <= 500.0)) {
    return pol_b3_0 + pol_b3_1*adc + pol_b3_2*(pow(adc,2)) + pol_b3_3*(pow(adc,3));
  }
  if ((500.0 < adc) && (adc <= 900.0)) {
    return pol_c3_0 + pol_c3_1*adc + pol_c3_2*(pow(adc,2)) + pol_c3_3*(pow(adc,3));
  }
  //if 900.0 < adc && adc <= 2047.0 {
  if ((900.0 < adc) && (adc <= 1600.0)) {
    return pol_d3_0 + pol_d3_1*adc + pol_d3_2*(pow(adc,2)) + pol_d3_3*(pow(adc,3));
  }
  return 0.0;
}
  
/// First derivative of the transfer function
auto g::TrackerStripTransferFunction::derivative(f32 adc) const -> f32 {
  if (adc < 0.0) {
    return 0.0;
  }
  if (adc <= 190.0) {
    return pol_a2_1 + 2.0*pol_a2_2*adc;
  }
  if ((190.0 < adc) && (adc <= 500.0)) {
    return pol_b3_1 + 2.0*pol_b3_2*adc + 3.0*pol_b3_3*(pow(adc,2));
  }
  if ((500.0 < adc) && (adc <= 900.0)) {
    return pol_c3_1 + 2.0*pol_c3_2 + 3.0*pol_c3_3*(pow(adc,2));
  }
  //if 900.0 < adc && adc <= 2047.0 {
  if ((900.0 < adc) && (adc <= 1600.0)) {
    return pol_d3_1 + 2.0*pol_d3_2 + 3.0*pol_d3_3*(pow(adc,2));
  }
  return 0.0;
}

// Create the inverse of the transfer function, which then 
// can be used in the simulation
// FIXME - switch to f64, however, only store f32 in the db
auto g::TrackerStripTransferFunction::invert(f64 y, f64 epsilon) -> Result<f64, g::AnalysisError> {
  f64 x = 0;
  f64 x_min = 0;
  f64 x_max = 1600;
  f64 y_min = evaluate(x_min);
  f64 y_max = evaluate(x_max);
  if ((y < y_min) || (y > y_max)){
    auto message = std::format("y value of {} is out of bounds for 0 <= x <= 1600", y);
    spdlog::error(message);
    auto err = g::AnalysisError(g::AnalysisError::ErrorKind::OutOfBounds, message);
    return Err(err);
  }
  u32 max_iterations = 50;

  f64 low  = x_min;
  f64 high = x_max;
  x = 0.5 * (low + high);
  for (u32 n=0;n<max_iterations;n++) {
    f64 f_x  = evaluate(x) - y;
    f64 df_x = derivative(x);
    // If we are close enough, or if the derivative is effectively zero
    if (std::abs(f_x) < epsilon) {
      return Ok(x);
    }
    // Update bisection bounds
    if (f_x > 0.0) {
      high = x;
    } else {
      low = x;
    }
    // Attempt a Newton-Raphson step
    if (std::abs(df_x) > 1e-9) {
      f64 next_x = x - f_x / df_x;
      // Safety check: if Newton step jumps outside our bounded bracket,
      // fallback to a safe Bisection step.
      if ((next_x > low) && (next_x < high)) {
        x = next_x;
        continue;
      }
    }
    // Fallback to Bisection if Newton fails or is too slow
    x = 0.5 * (low + high);
  }
  auto message = std::format("We couldn't find a solution whcih is epsilon {} away! Try to reduce epsilon", epsilon);
  spdlog::error(message);
  auto err = g::AnalysisError(g::AnalysisError::ErrorKind::DidNotConverge, message);
  return Err(err);
}



namespace gondola {
  std::ostream& operator<<(std::ostream& os, const g::TrackerStripTransferFunction& tf) {
    os << tf.to_string();
    return os;
  }
}


#endif
