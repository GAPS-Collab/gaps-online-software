/// This file is part of gaps-online-software and published
/// under the GPLv3 license
#pragma once

#include "gondola.hpp"

namespace gondola {

  /// Corrections applied post-flight to quantities that were computed
  /// online without access to the full raw waveform (e.g. a fast
  /// baseline rms estimate). Each function here undoes a specific,
  /// known bias of its no-waveform online counterpart.

  /// Correct a no-waveform baseline rms estimate for the bias
  /// introduced by a nonzero mean mu over a window of num samples.
  ///
  /// Implemented in src/events.cxx
  ///
  /// @param mu   : online-estimated mean of the baseline window
  /// @param sigB : online-estimated (biased) rms of the baseline window
  /// @param num  : number of samples the online estimate was taken over
  ///
  /// @returns the corrected rms, or NaN if the correction is undefined
  ///          (negative radicand or non-finite intermediate value)
  auto corrected_rms_noWF(f32 mu, f32 sigB, u32 num = 200) -> f32;
}
