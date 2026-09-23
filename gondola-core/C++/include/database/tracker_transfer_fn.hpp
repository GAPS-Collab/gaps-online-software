#pragma once

#include "result/result.h"
#include "gondola_typedefs.hpp"
#include "errors.hpp"

namespace r = result;

namespace gondola {
  struct TrackerStripTransferFunction {  
    i32          data_id            ; 
    i32          strip_id           ; 
    i64          volume_id          ; 
    i64          utc_timestamp_start; 
    i64          utc_timestamp_stop ; 
    std::string  name               ; 
    f32          pol_a2_0           ; 
    f32          pol_a2_1           ; 
    f32          pol_a2_2           ; 
    f32          pol_b3_0           ; 
    f32          pol_b3_1           ; 
    f32          pol_b3_2           ; 
    f32          pol_b3_3           ; 
    f32          pol_c3_0           ; 
    f32          pol_c3_1           ; 
    f32          pol_c3_2           ; 
    f32          pol_c3_3           ; 
    f32          pol_d3_0           ; 
    f32          pol_d3_1           ; 
    f32          pol_d3_2           ; 
    f32          pol_d3_3           ; 
  
    /// The actual transfer function for this 
    /// strip. Calculate energy from adc values
    /// @param: adc - the actual adc value for this 
    ///               strip. THe true value is an 
    ///               unsigned integer
    auto evaluate(f32 adc) const -> f32;
  
    auto derivative(f32 adc) const -> f32;
    
    auto to_string() const -> std::string;

    auto invert(f64 y, Option<f64> epsilon_opt) -> r::Result<f64, AnalysisError>;
    
    friend std::ostream& operator<<(std::ostream& os, const TrackerStripTransferFunction& tfn);
  }; 
}
