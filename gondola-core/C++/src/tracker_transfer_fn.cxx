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

//----------------------------------------------------------------------------

auto g::TrackerStripTransferFunction::to_string() const -> std::string {
  auto repr = std::format("<TrackerStripTransferFunction [{}]:", strip_id);
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

//----------------------------------------------------------------------------
  
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

//----------------------------------------------------------------------------
  
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
    return pol_c3_1 + 2.0*pol_c3_2*adc + 3.0*pol_c3_3*(pow(adc,2));
  }
  //if 900.0 < adc && adc <= 2047.0 {
  if ((900.0 < adc) && (adc <= 1600.0)) {
    return pol_d3_1 + 2.0*pol_d3_2*adc + 3.0*pol_d3_3*(pow(adc,2));
  }
  return 0.0;
}

//----------------------------------------------------------------------------

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
    auto message = std::format("y value of {} is out of bounds for {} <= y <= {} [ for adc in 0 .. 1600]", y, y_min, y_max);
    SPDLOG_ERROR(message);
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
  SPDLOG_ERROR(message);
  auto err = g::AnalysisError(g::AnalysisError::ErrorKind::DidNotConverge, message);
  return Err(err);
}

//============================================================================ 

auto g::get_trkstriptransferfn() -> TrkStripTransferFnMap {
  TrkStripTransferFnMap tf_map;
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return tf_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstriptransferfunction",
      make_column("data_id"              , &g::TrackerStripTransferFunction::data_id, primary_key()),
      make_column("strip_id"             , &g::TrackerStripTransferFunction::strip_id), 
      make_column("volume_id"            , &g::TrackerStripTransferFunction::volume_id), 
      make_column("utc_timestamp_start"  , &g::TrackerStripTransferFunction::utc_timestamp_start), 
      make_column("utc_timestamp_stop"   , &g::TrackerStripTransferFunction::utc_timestamp_stop),  
      make_column("name"                 , &g::TrackerStripTransferFunction::name),
      make_column("pol_a2_0"             , &g::TrackerStripTransferFunction::pol_a2_0),
      make_column("pol_a2_1"             , &g::TrackerStripTransferFunction::pol_a2_1),
      make_column("pol_a2_2"             , &g::TrackerStripTransferFunction::pol_a2_2),
      make_column("pol_b3_0"             , &g::TrackerStripTransferFunction::pol_b3_0),
      make_column("pol_b3_1"             , &g::TrackerStripTransferFunction::pol_b3_1),
      make_column("pol_b3_2"             , &g::TrackerStripTransferFunction::pol_b3_2),
      make_column("pol_b3_3"             , &g::TrackerStripTransferFunction::pol_b3_3),
      make_column("pol_c3_0"             , &g::TrackerStripTransferFunction::pol_c3_0),
      make_column("pol_c3_1"             , &g::TrackerStripTransferFunction::pol_c3_1),  
      make_column("pol_c3_2"             , &g::TrackerStripTransferFunction::pol_c3_2),  
      make_column("pol_c3_3"             , &g::TrackerStripTransferFunction::pol_c3_3),  
      make_column("pol_d3_0"             , &g::TrackerStripTransferFunction::pol_d3_0),  
      make_column("pol_d3_1"             , &g::TrackerStripTransferFunction::pol_d3_1),  
      make_column("pol_d3_2"             , &g::TrackerStripTransferFunction::pol_d3_2), 
      make_column("pol_d3_3"             , &g::TrackerStripTransferFunction::pol_d3_3)
  ));  
  auto tfs = storage.get_all<g::TrackerStripTransferFunction>();
  for (auto const &tf : tfs) {
    tf_map.insert({tf.strip_id, tf});
  }  
  return tf_map;
}    

//============================================================================ 

auto g::get_trkstriptransferfn_by_volumeid() -> TrkStripTransferFnMap {
  TrkStripTransferFnMap hw_map = get_trkstriptransferfn();
  TrkStripTransferFnMap tf_map; 
  for (auto const &[hwid, s] : hw_map) {
    tf_map.insert(std::make_pair(s.volume_id, s)); 
  }
  return tf_map;
}    

//============================================================================ 

namespace gondola {
  std::ostream& operator<<(std::ostream& os, const g::TrackerStripTransferFunction& tf) {
    os << tf.to_string();
    return os;
  }
}


#endif
