/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once

#include <span>

#include "result/result.h"
namespace r = result;

#include "gondola_typedefs.hpp"
#include "errors.hpp"

namespace gondola {
    
  [[nodiscard("CFD time extraction can fail")]]
  auto find_cfd_simple_time(
      std::span<const f64> voltages,
      std::span<const f64> times_ns,
      f64 cfd_fraction,
      usize peak_begin,
      usize peak_end
  ) -> r::Result<f64, AnalysisError>;
}




