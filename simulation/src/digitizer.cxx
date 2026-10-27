#include <cstdlib>
#include <pthread.h>

#include "database.h"
#include "io.hpp"
#include "packets/tof_packet.h"
#include "tof_algorithms.hpp"

#include "digitizer.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g = gondola;
using namespace result;

//----------------------------------------------------------

constexpr f64 DRS4_MIN_MV = -250.0;
constexpr f64 DRS4_MAX_MV =  750.0;
constexpr f64 TOF_SATURATION_MV = 740.0;
constexpr f64 TOF_SAT_TOT_THRESHOLD_MV = 550.0;

//----------------------------------------------------------


namespace { // anonymus namespace ensures the following are only 
            // defined in this compliation unit

  struct SatTotCalibCoeff {
    f64 ws;
    f64 rss;
    f64 rsc;
  };
  
  auto sat_tot_coeffs_for_position(
      f64 position_centered_mm,
      f64 paddle_length_mm
  ) -> SatTotCalibCoeff {
    // Middle region by default.
    SatTotCalibCoeff coeff{7.86162, 1.15745, 484.273};
  
    if (paddle_length_mm <= 0.0) {
      return coeff;
    }
  
    const f64 half = paddle_length_mm / 2.0;
    const f64 span = paddle_length_mm / 6.0;
  
    if (position_centered_mm <= -half + span) {
      coeff = {12.2485, 1.01828, 467.976};
    } else if (position_centered_mm >= half - span) {
      coeff = {3.0275, 0.610848, 569.51};
    }
  
    return coeff;
  }
  
  //auto time_over_threshold(
  //    const Vec<f64>& time_ns,
  //    const Vec<f64>& voltage_mv,
  //    f64 threshold_mv
  //) -> f64 {
  //  const usize n = std::min(time_ns.size(), voltage_mv.size());
  //
  //  usize first = n;
  //  usize last = n;
  //
  //  for (usize i = 0; i < n; ++i) {
  //    if (voltage_mv[i] > threshold_mv) {
  //      if (first == n) {
  //        first = i;
  //      }
  //      last = i;
  //    }
  //  }
  //
  //  if (first == n) {
  //    return -1.0;
  //  }
  //
  //  return time_ns[last] - time_ns[first];
  //}
 template <typename TimeContainer, typename VoltageContainer>
 auto time_over_threshold(
    const TimeContainer& time_ns,
    const VoltageContainer& voltage_mv,
    f64 threshold_mv
  ) -> f64 {
    const usize n = std::min(time_ns.size(), voltage_mv.size());

    usize first = n;
    usize last = n;

    for (usize i = 0; i < n; ++i) {
        if (voltage_mv[i] > threshold_mv) {
            if (first == n) {
                first = i;
            }
            last = i;
        }
    }

    if (first == n) {
        return -1.0;
    }

    return time_ns[last] - time_ns[first];
  } 
 
  template <typename TimeContainer, typename VoltageContainer>
  auto rise_slope_at_threshold(
    const TimeContainer& time_ns,
    const VoltageContainer& voltage_mv,
    f64 threshold_mv
  ) -> f64 {
    const usize n = std::min(time_ns.size(), voltage_mv.size());

    if (n < 6) {
        return 0.0;
    }

    const f64 threshold = std::abs(threshold_mv);

    for (usize i = 1; i < n; ++i) {
        if (std::abs(voltage_mv[i]) < threshold) {
            continue;
        }

        const usize idx = i - 1;

        if (idx < 3 || idx + 2 >= n) {
            return 0.0;
        }

        const f64 above =
            (voltage_mv[idx] +
             voltage_mv[idx + 1] +
             voltage_mv[idx + 2]) / 3.0;

        const f64 below =
            (voltage_mv[idx - 1] +
             voltage_mv[idx - 2] +
             voltage_mv[idx - 3]) / 3.0;

        const f64 dt = time_ns[idx + 1] - time_ns[idx - 2];

        return dt != 0.0 ? (above - below) / dt : 0.0;
    }

    return 0.0;
  } 
 //auto rise_slope_at_threshold(
 //     const Vec<f64>& time_ns,
 //     const Vec<f64>& voltage_mv,
 //     f64 threshold_mv
 // ) -> f64 {
 //   const usize n = std::min(time_ns.size(), voltage_mv.size());
 // 
 //   if (n < 6) {
 //     return 0.0;
 //   }
 // 
 //   const f64 threshold = std::abs(threshold_mv);
 // 
 //   for (usize i = 1; i < n; ++i) {
 //     if (std::abs(voltage_mv[i]) < threshold) {
 //       continue;
 //     }
 // 
 //     const usize idx = i - 1;
 // 
 //     if (idx < 3 || idx + 2 >= n) {
 //       return 0.0;
 //     }
 // 
 //     const f64 above =
 //         (voltage_mv[idx] +
 //          voltage_mv[idx + 1] +
 //          voltage_mv[idx + 2]) / 3.0;
 // 
 //     const f64 below =
 //         (voltage_mv[idx - 1] +
 //          voltage_mv[idx - 2] +
 //          voltage_mv[idx - 3]) / 3.0;
 // 
 //     const f64 dt = time_ns[idx + 1] - time_ns[idx - 2];
 // 
 //     return dt != 0.0 ? (above - below) / dt : 0.0;
 //   }
 // 
 //   return 0.0;
 // }
  
  auto template_time_axis(
      const g::TofPulseTemplate& pulse
  ) -> Vec<f64> {
    Vec<f64> time_ns(pulse.mean.size());
  
    for (usize i = 0; i < time_ns.size(); ++i) {
      time_ns[i] = pulse.time_start +
                   static_cast<f64>(i) * pulse.time_step;
    }
  
    return time_ns;
  }
  
  auto adjust_saturated_pulse_width(
      Vec<f64>& voltage_mv,
      Vec<f64>& rms_mv,
      f64 time_step_ns,
      f64 delta_width_ns,
      std::mt19937& rng
  ) -> void {
    constexpr f64 saturation_mv = TOF_SATURATION_MV;
    constexpr f64 sat_fuzz_mv = 3.0;
  
    if (voltage_mv.empty() ||
        voltage_mv.size() != rms_mv.size() ||
        !(time_step_ns > 0.0) ||
        !std::isfinite(delta_width_ns)) {
      return;
    }
  
    const int n = static_cast<int>(voltage_mv.size());
  
    int lo = -1;
    int hi = -1;
  
    for (int i = 0; i < n; ++i) {
      if (voltage_mv[i] >= saturation_mv - sat_fuzz_mv) {
        if (lo < 0) {
          lo = i;
        }
        hi = i;
      }
    }
  
    if (delta_width_ns > 0.0 && hi >= 0) {
      int bins = static_cast<int>(
          delta_width_ns / time_step_ns + 0.5
      );
      bins = std::max(1, bins);
  
      std::normal_distribution<f64> plateau_noise(
          0.0, sat_fuzz_mv / 2.0
      );
  
      for (int i = n - 1; i > hi; --i) {
        const int src = i - bins;
  
        if (src > hi) {
          voltage_mv[i] = voltage_mv[src];
          rms_mv[i] = rms_mv[src];
        } else {
          voltage_mv[i] =
              saturation_mv + plateau_noise(rng);
          rms_mv[i] = sat_fuzz_mv / 2.0;
        }
      }
    } else if (delta_width_ns < 0.0 && hi > lo + 2) {
      int bins = static_cast<int>(
          -delta_width_ns / time_step_ns + 0.5
      );
      bins = std::max(1, bins);
      bins = std::min(bins, hi - lo - 1);
  
      int erase_start = hi - bins;
      erase_start = std::max(erase_start, lo + 1);
  
      const int shift = hi - erase_start;
  
      if (shift >= 1) {
        for (int i = erase_start; i + shift < n; ++i) {
          voltage_mv[i] = voltage_mv[i + shift];
          rms_mv[i] = rms_mv[i + shift];
        }
  
        const f64 fill_value =
            n > shift ? voltage_mv[n - shift - 1] : 0.0;
  
        for (int i = n - shift; i < n; ++i) {
          voltage_mv[i] = fill_value;
          rms_mv[i] = 0.0;
        }
      }
    }
  }
} // namespace


//----------------------------------------------------------

auto g::find_tof_waveform_peak(const g::SimTofWaveform& waveform) -> Option<g::SimTofWaveformPeak> {
  if (waveform.voltage_mv.empty()) {
    return None;
  }
  const auto peak_it = std::max_element(waveform.voltage_mv.begin(), waveform.voltage_mv.end());
  const auto peak_index = static_cast<std::size_t>(std::distance(waveform.voltage_mv.begin(), peak_it));
  auto peak = g::SimTofWaveformPeak();
  peak.voltage_mv   = *peak_it;
  peak.time_ns      = waveform.time_at(peak_index);
  peak.sample_index = peak_index;
  return peak;
}

//----------------------------------------------------------
  
auto g::integrate_tof_waveform(const g::SimTofWaveform& waveform, f64 impedance_ohm) -> f64 {
  const f64 voltage_sum = std::accumulate(waveform.voltage_mv.begin(), waveform.voltage_mv.end(), 0.0);
  return (voltage_sum * waveform.time_step_ns)  / impedance_ohm;
}

//----------------------------------------------------------

auto g::extract_tof_waveform_features(const g::SimTofWaveform& waveform, f64 cfd_fraction) -> r::Result<g::SimTofWaveformFeatures, AnalysisError> {
  const auto peak = g::find_tof_waveform_peak(waveform);
  if (!peak) {
    auto message = std::format("Input is invalid!");
    SPDLOG_ERROR(message);  
    auto err = g::AnalysisError(g::AnalysisError::ErrorKind::InvalidInput, message);
    return Err(err);
  }

  Vec<f64> times_ns(waveform.voltage_mv.size());

  for (std::size_t i = 0; i < times_ns.size(); ++i) {
    times_ns[i] = waveform.time_at(i);
  }

  // Keep the peak away from the first and last samples
  // because the CFD algorithm needs neighboring samples.
  const usize peak_begin = 10;
  const usize peak_end = waveform.voltage_mv.size();

  const auto cfd = g::find_cfd_simple_time(waveform.voltage_mv, times_ns, cfd_fraction,
                                        peak_begin,peak_end);

  if (cfd.is_err()) {
    auto message = std::format("Could not extract time from waveform for paddle {}!", waveform.paddle_id);
    SPDLOG_ERROR(message);  
    auto err = g::AnalysisError(g::AnalysisError::ErrorKind::CfdError, message);
    return Err(err);
  }

  const f64 integral = g::integrate_tof_waveform(waveform);

  auto features = g::SimTofWaveformFeatures();
  features.peak_voltage_mv = peak->voltage_mv;
  features.peak_time_ns    = peak->time_ns;
  features.cfd_time_ns     = cfd.try_ok();
  features.integral_mv_ns  = integral*50.0;
  features.charge_pc       = integral;
  return Ok(features);
}

//----------------------------------------------------------

// constructor
g::GapsDigitizer::GapsDigitizer(const SimConfig& cfg) : cfg(cfg){
  std::random_device rd;
  random_gen_.seed(rd());             ;
  // FIXME - instead of copying it all, we should have one 
  // field with the config -> Refactor
  use_digitizer            = cfg.digitization;
  tof_add_gauss_jitter     = cfg.digi_tof_add_gauss_jitter;
  // especially these fields are rather useless, since only used once
  //tof_voltage_width_       = cfg.digi_tof_voltage_width;
  //tof_voltage_offset_      = cfg.digi_tof_voltage_offset;
  //tof_timing_offset_       = cfg.digi_tof_timing_offset;
  //tof_timing_width_        = cfg.digi_tof_timing_width;
  //tof_pos_offset_          = cfg.digi_tof_pos_offset;
  //tof_pos_width_           = cfg.digi_tof_pos_width;
  //trk_energy_offset_       = cfg.digi_trk_energy_offset;
  //trk_energy_width_        = cfg.digi_trk_energy_width; 
  trk_add_cmn_noise_       = cfg.digi_trk_add_cmn_noise;
  tof_volt_smearing_       = std::normal_distribution<f64>(cfg.digi_tof_voltage_offset, cfg.digi_tof_voltage_width);
  tof_time_smearing_       = std::normal_distribution<f64>(cfg.digi_tof_timing_offset,  cfg.digi_tof_timing_width);
  tof_pos_smearing_        = std::normal_distribution<f64>(cfg.digi_tof_pos_offset,     cfg.digi_tof_pos_width);
  trk_energy_smearing_     = std::normal_distribution<f64>(cfg.digi_trk_energy_offset,  cfg.digi_trk_energy_width);
  paddles_                 = g::get_tofpaddles_by_volumeid();
  strips_                  = g::get_trackerstrips_by_volumeid();
  strip_tfns_              = g::get_trkstriptransferfn_by_volumeid(cfg.digi_trk_tf_name);
  spdlog::info("Loaded {} TRK transfer fns with name = {}", strip_tfns_.size(), cfg.digi_trk_tf_name);
  strip_pedestals_         = g::get_trackerstrippedestals_by_volumeid(cfg.digi_trk_ped_name);
  spdlog::info("Loaded {} TRK pedestals with name = {}", strip_pedestals_.size(), cfg.digi_trk_ped_name);
  strip_mask_              = g::get_trackerstripmasks(cfg.digi_trk_mask_name); 
  strip_vid_hid_map_       = g::get_vid_hid_map_tracker();
  auto strip_hid_vid_map   = g::get_hid_vid_map_tracker(); // we only need this once 
  g::TrkStripMaskMap temp_mask;
  for (const auto &[hid, m] : strip_mask_) {
    u32 vol_id = strip_hid_vid_map[hid];
    temp_mask.insert({vol_id, m});
  }
  strip_mask_ = temp_mask;
  spdlog::info("Loaded {} TRK masks with name = {}", strip_mask_.size(), cfg.digi_trk_mask_name);
}

//----------------------------------------------------------

g::GapsDigitizer::~GapsDigitizer() {
  print_statistics();
}

//----------------------------------------------------------

auto g::GapsDigitizer::to_string() const -> std::string {
  std::string repr = "<GapsDigitizer>";
  return repr;
}

//----------------------------------------------------------

auto g::GapsDigitizer::print_statistics() -> void {
  const auto attempted       = tof_hits_attempted;
  const auto succeeded       = tof_hits_succeeded;
  const auto failed          = tof_hits_failed;
  const auto below_threshold = tof_hits_below_th;

  const double denominator        = attempted > 0 ? static_cast<f64>(attempted) : 1.0;
  const double denominator_failed = failed > 0 ? static_cast<f64>(failed) : 1.0;
  const usize all_wf              = wf_no_tof_pulse_found + wf_voltage_empty;
  const double denominator_wf     = all_wf > 0 ? static_cast<f64>(all_wf) : 1.0;
  const usize all_templ_finding   = expected_peak_mv_not_finite + no_template_for_pid; 
  const double denominator_templ  = all_templ_finding > 0 ? static_cast<f64>(all_templ_finding) : 1.0; 
  const usize all_voltage_empty   = voltage_empty_max_el_invalid + voltage_empty_expected_peak_invalid; 
  const double denominator_volt   = all_voltage_empty > 0 ? static_cast<f64>(all_voltage_empty) : 1.0; 
  const double denominator_p_inv  = voltage_empty_expected_peak_invalid > 0 ? static_cast<f64>(voltage_empty_expected_peak_invalid) : 1.0;
  const double denominator_p_neg  = volt_empty_exp_peak_neg > 0 ? static_cast<f64>(volt_empty_exp_peak_neg) : 1.0;
  const double denominator_mc     = n_total_mc_hits > 0 ? static_cast<f64>(n_total_mc_hits) : 1.0;  
  const double denominator_sat    = tof_hits_sat > 0 ? static_cast<f64>(tof_hits_sat) : 1.0;
  const double denominator_fb     = tof_hits_digi_fallback > 0 ? static_cast<f64>(tof_hits_digi_fallback) : 1.0; 
  auto repr = std::format(
    "[TofDigitizerStats] (total events {}) \n"
    "  attempted: {} | "
    "succeeded: {} ({:.2f}%) | "
    "failed: {} ({:.2f}%) | "
    "below threshold: {} ({:.2f}%) | \n"
    "  failed reason wf: {} ({:.2f}%) | "
    "failed reason no feat: {} ({:.2f}%)\n" 
    "  wf failed no pulse: {} ({:.2f}%) | "
    "wf failed voltage 0: {} ({:.2f}%)\n"
    "  templ peak finding fail infinite : {} ({:.2f}%) | "
    "templ peak finding fail no pid : {} ({:.2f}%)\n"
    "  empty volt - expected peak inval : {} ({:.2f}%) | " 
    "empty volt - max element inval : {} ({:.2f}%)\n"
    "  exp peak is NAN                  : {} ({:.2f}%) | "
    "exp peak is <= 0               : {} ({:.2f}%) | "
    "exp peak is INF  : {} ({:.2f}%)\n"
    "  exp peak input 0                 : {} ({:.2f}%) | "
    "hits with E == 0               : {}\n"
    "  mc hits                          : {} | "
    "mc hits E == 0                 : {} ({:.2f}%)\n" 
    "  hits excluded due masking        : {}"
    "  saturated TOF hits               : {}" 
    "  sat TOF hits no templ found      : {} ({:.2f}%)\n"
    "fallbacks                     : {} \n"
    "  fallbacks with E==0              : {} ({:.2f}%)\n",
    n_events,
    attempted,
    succeeded, 100.0 * succeeded / denominator,
    failed, 100.0 * failed / denominator,
    below_threshold, 100.0 * below_threshold / denominator,
    n_failed_reason_wf_bad, 100.0* n_failed_reason_wf_bad / denominator_failed, 
    n_failed_reason_no_features, 100* n_failed_reason_no_features / denominator_failed, 
    wf_no_tof_pulse_found, 100.0* wf_no_tof_pulse_found / denominator_wf,
    wf_voltage_empty, 100.0* wf_voltage_empty / denominator_wf,
    expected_peak_mv_not_finite, 100.0*expected_peak_mv_not_finite / denominator_templ,
    no_template_for_pid, 100.0*no_template_for_pid / denominator_templ,
    voltage_empty_expected_peak_invalid, 100.0*voltage_empty_expected_peak_invalid / denominator_volt,
    voltage_empty_max_el_invalid, 100.0*voltage_empty_max_el_invalid / denominator_volt,
    volt_empty_exp_peak_nan, 100.0*volt_empty_exp_peak_nan / denominator_p_inv ,
    volt_empty_exp_peak_neg, 100.0*volt_empty_exp_peak_neg / denominator_p_inv ,    
    volt_empty_exp_peak_inf, 100.0*volt_empty_exp_peak_inf / denominator_p_inv , 
    energy_mev_zero, 100.0*energy_mev_zero / denominator_p_neg,
    zero_energy_hits,
    n_total_mc_hits,
    n_total_mc_hits_zero_en, 100.0*n_total_mc_hits_zero_en / denominator_mc,
    tof_hits_excluded_rb_mask,
    tof_hits_sat,
    tof_hits_sat_no_templ_found, 100.0*tof_hits_sat_no_templ_found /denominator_sat,
    tof_hits_digi_fallback, tof_hits_digi_fallback_e0, 100.0*tof_hits_digi_fallback_e0 / denominator_fb
  );
  std::cout << repr << std::endl;
}

//----------------------------------------------------------

auto g::GapsDigitizer::integrate_mc_hits(const Vec<g::McHit>& hits, f64 min_time_delta) -> Vec<g::RecoHit> {
  // first sort hits by volume 
  HashMap<u32, Vec<g::McHit>> volume_to_hits;
  for (auto const &h : hits) {
    if (h.step_edep == 0) {
      continue; // in the original code, this is done in event action
    }
    if (volume_to_hits.contains(h.volume_id)) {
      volume_to_hits[h.volume_id].push_back(h);
    } else {
      volume_to_hits.insert(std::pair<u32, Vec<McHit>>(h.volume_id, Vec<McHit>(1, h)));
    }
  }
  Vec<g::RecoHit> reco_hits;
  for (const auto& [vol, vol_hits] : volume_to_hits) {
    for (const auto& h : vol_hits) {
      auto reco_hit   = g::RecoHit();
      reco_hit.x      = h.pos_x;
      reco_hit.y      = h.pos_y;
      reco_hit.z      = h.pos_z;
      reco_hit.volume = h.volume_id; 
      reco_hit.energy = h.step_edep;
      reco_hit.time   = h.glob_time;
      reco_hits.push_back(reco_hit);
    }
  }
  //std::cout << "Got " << hits.size() << " mc hits and " << reco_hits.size() << " reco hits!" << std::endl;
  reco_hits = g::GapsDigitizer::merge_hits(reco_hits);
  return reco_hits;
}

//----------------------------------------------------------

auto g::GapsDigitizer::merge_hits(
    const Vec<g::RecoHit>& hits,
    f64 min_time_delta
) -> Vec<g::RecoHit> {
  min_time_delta = std::abs(min_time_delta);
  if (hits.empty()) {
      return {};
  }
  // Work on a copy so we don't modify the input.
  auto sorted_hits = hits;
  std::sort(
      sorted_hits.begin(),
      sorted_hits.end(),
      [](const auto& a, const auto& b) {
          if (a.volume != b.volume) {
              return a.volume < b.volume;
          }
          return a.time < b.time;
      }
  );

  Vec<g::RecoHit> merged_hits;
  merged_hits.reserve(hits.size());
  
  auto merged_hit = sorted_hits.front();
  auto window_start = merged_hit.time;

  for (usize idx = 1; idx < sorted_hits.size(); ++idx) {
    const auto& hit = sorted_hits[idx];

    const bool same_volume = hit.volume == merged_hit.volume;
    const bool inside_time_window = hit.time - window_start <= min_time_delta;

    if (same_volume && inside_time_window) {
      merged_hit = merged_hit + hit;
    } else {
      merged_hits.push_back(std::move(merged_hit));
      merged_hit = hit;
      window_start = hit.time;
    }
  }
  // Don't forget the final group.
  merged_hits.push_back(std::move(merged_hit));
  return merged_hits;
}


//----------------------------------------------------------

///// Detector response for the tof
//auto g::GapsDigitizer::tof_response_fn_simple(g::RecoHit& hit) -> void {
//  // timing, position, energy 
//  hit.time             = tof_timing_response_fn_simple(hit.time);
//  f64 pos_along_paddle = tof_pos_response_fn_simple(hit);
//  hit.energy           = tof_energy_response_fn_simple(hit.energy, pos_along_paddle);
//}

//----------------------------------------------------------
    
auto g::GapsDigitizer::tof_energy_response_fn_simple(f64 energy, f64 pos_along_paddle) -> f64 {
  // this is taken over from SimpleDet directly
  f64 vpeak       = 35.26*energy / (1+0.021*energy); // Birks' Law
  f64 xdependence = 1.044*pow((pos_along_paddle+900.)/1800.,2.) + 0.739;
  vpeak *= xdependence;
  vpeak += tof_volt_smearing_(random_gen_);
  //vpeak += random_generator_.Gaus(0, par.VoltageWidth);
  //if (vpeak > 950) // DRS4 saturation - may not actually degrade energy res.
  //  vpeak = 950;
  if (vpeak > 1356*xdependence) {
    // Birks' Law saturation - 1356 = vpeak@200MeV
    return 200;
  }
  // get corresponding energy
  double reco_energy = vpeak/(35.26*xdependence-0.021*vpeak);
  return (reco_energy > 0) ? reco_energy : 0; // force non-negative value
}

//----------------------------------------------------------
    
auto g::GapsDigitizer::tof_timing_response_fn_simple(f64 time) -> f64 {
  f64 time_shift = 1.0;
  f64 reco_time  = time*time_shift;
  reco_time += tof_time_smearing_(random_gen_);
  return (reco_time > 0) ? reco_time : 0;
}

//----------------------------------------------------------
    
//auto g::GapsDigitizer::tof_pos_response_fn_simple(RecoHit& hit) -> f64 {
//  f64 pos_along_paddle = 0;
//  // get the relevant paddle from our list 
//  // this is needed to find the principal axes 
//  // of the paddle
//  auto paddle   = paddles_[hit.volume];  
//  auto princ    = paddle.get_principal(); 
//  Vec<f64> pos_on_paddle = {hit.x, hit.y, hit.z};
//  Vec<f64> pos_paddle_a_side = {paddle.global_pos_x_l0_A,
//                                paddle.global_pos_y_l0_B,
//                                paddle.global_pos_z_l0_A};
//  // parameter, which I think should never be used 
//  f64 pos_scale = 0;
//  for (usize k=0; k<3; k++) {
//    pos_on_paddle[k] *= princ[k];
//    // is there a principal component? 
//    // float comparison, use 0.1 as random epsilon
//    if (pos_on_paddle[k] > 0.1) {
//      pos_on_paddle[k] *= pos_scale;
//      pos_on_paddle[k] += tof_pos_smearing_(random_gen_); 
//      pos_along_paddle  = pos_on_paddle[k] - pos_paddle_a_side[k]; 
//      if (k == 0) {
//        hit.x = pos_on_paddle[k];
//      }
//      if (k == 1) {
//        hit.y = pos_on_paddle[k];
//      }
//      if (k == 2) {
//        hit.z = pos_on_paddle[k];
//      }
//      break;
//    }
//  }
//  return pos_along_paddle;
//}

//----------------------------------------------------------

/// Detector response for the tracker
auto g::GapsDigitizer::trk_response_fn(g::RecoHit& hit) -> u32 {
  // two things - position and energy. 
  // The position is simply the strip position, no magick here 
  auto strip = strips_[hit.volume];
  hit.x = strip.global_pos_x_l0;
  hit.y = strip.global_pos_y_l0;
  hit.z = strip.global_pos_z_l0;
  f64 energy = hit.energy;
  hit.energy += trk_energy_smearing_(random_gen_);
  if (hit.energy < 0) {
    // cut off at 0
    hit.energy = 0;
  }
  f64 simu_V = energy/.841;
  // get the transfer function for this strip 
  auto tf  = strip_tfns_[hit.volume];
  // to get to adc, we need to invert it 
  f32 adc_float  = 0;
  auto adc_res = tf.invert(hit.energy*1000, 1e-2);
  if (!(adc_res.is_ok())) {
    // reduce epsilon and try again 
    //spdlog::warn("Inversion of TF is not converging, reducing epsilon...");
    auto adc_res_2 = tf.invert(hit.energy*1000, 1e-2);
    if (!(adc_res_2.is_ok())) {
      spdlog::error("Unable to invert TF for strip {}", tf.strip_id);
      return 0; 
    } else {
      adc_float = adc_res_2.unwrap();
    }
  } else {
    adc_float = adc_res.unwrap();
  }  
  i32 adc = std::round(adc_float);
  if (adc < 0) {
    spdlog::warn("Got adc < 0");
  }
  // get the pedestal width for this strip 
  // and use it to jitter the adc around a litttle bit
  f64 res = strip_pedestals_[hit.volume].pedestal_sigma;
  // we subtract the pedestal in L1, so we have to add it here 
  f64 mn  = strip_pedestals_[hit.volume].pedestal_mean;
  // I think in the original, we have 
  // f64 mn = 0 here
  auto ped_dist  = std::normal_distribution<f64>(mn, res);
  adc += std::round(ped_dist(random_gen_));
  if (adc < 0) {
    adc = 0;
  }
  
  //Zhen's common noise
  //if (fUseCMN && !input_is_adc) {
  //  uint mkey = volumeid / 10000;  // module key
  //  auto it = fEventCMNLevel.find(mkey);
  //  if (it != fEventCMNLevel.end()) {
  //    double channelGain = GetGAIN(volumeid);
  //    double CMN = it->second * channelGain;  // 
  //    disc_adc += CMN;
  //    if (energy < 0) energy = 0;
  //    log_trace("CMN: volid=" << volumeid << " module=" << mkey
  //              << " level=" << it->second << " gain=" << channelGain
  //              << " noise=" << CMN << " keV, final=" << energy << " keV");
  //  } else {
  //    log_warn("CMN: no level found for module " << mkey << " (vid=" << volumeid << ")");
  //  }
  //}
  // Saturate to the maximum value (11 bit = 2^11)
  if(adc > std::pow(2, 11)) {
    spdlog::warn("Got adc > 2**11 (dynamic range of ASIC)");
    adc = std::pow(2, 11);
  }
  // after the rounding and clipping, we convert back to the 
  // energy (admittedly the above steps where kind of minimal)
  hit.energy = (tf.evaluate(adc)*.841)/1000; // mV to KeV to MeV
  return (u32)adc;
}

//----------------------------------------------------------
    
auto g::GapsDigitizer::tof_trigger(const Vec<g::RecoHit>& hits) const -> bool {
  return true;
}

//----------------------------------------------------------

auto g::GapsDigitizer::digitize_event(const g::McEvent& mc_ev, bool& check_trigger) -> Option<g::TelemetryEvent> {
  g::TelemetryEvent ev;
  ev.tof_event.event_id = mc_ev.event_id;
  ev.event_id           = mc_ev.event_id;
  n_total_mc_hits += mc_ev.hits.size(); 
  for (auto const &h : mc_ev.hits) {
    if (h.step_edep == 0) { 
      n_total_mc_hits_zero_en++;
    }
  }
  auto r_hits = g::GapsDigitizer::integrate_mc_hits(mc_ev.hits, 1e20);
  
  //if (r_hits.size() > 0) {
  //  //std::cout << "DIGITIZER GOT " << r_hits.size() << " RECO HITS AND " << mc_ev.hits.size() << " MC HITS" << std::endl;
  //  std::cout << "------------------------------------" << std::endl;
  //}
  for (auto& h : r_hits) {
    //std::cout << h.volume << std::endl;
    if (h.volume < 200000000) {
      if(h.energy == 0.0) {
        zero_energy_hits++;
      }
      if (std::find(cfg.digi_tof_excluded_readout.begin(),cfg.digi_tof_excluded_readout.end(),h.volume)
          != cfg.digi_tof_excluded_readout.end()) {
        tof_hits_excluded_rb_mask++;
        continue;
      }
      //tof_response_fn_simple(h);
      auto digi_hit_res = digitize_tof_hit(h);
      tof_hits_attempted++;
      if (tof_hits_attempted % 10000 == 0) {
        print_statistics();
      }
      if (digi_hit_res.is_some()) {
        ev.tof_event.hits.push_back(digi_hit_res.unwrap());
        ++tof_hits_succeeded;
      } else {
        if (cfg.digi_use_tof_legacy_fb) {
          auto fallback_result = digitize_tof_hit_fallback(h);
          tof_hits_digi_fallback++;  
          if (fallback_result.is_some()) {
            auto fallback_hit = fallback_result.unwrap();

            if (fallback_hit.peak_a_f32 == 0.0f) {
              //++tof_hits_fallback_zero_energy;
            }

            ev.tof_event.hits.push_back(fallback_hit);
            tof_hits_failed++;
          }
        }
      }
    } else {
      // first step is to check if we actually have this strip 
      if (!strip_mask_.contains(h.volume)) {
        continue;
      }
      // if it has, this strip might be dead
      if (!strip_mask_[h.volume]) {
        continue;
      }
      //continue;
      //std::cout << "VOLUME ID " << h.volume << " " << std::endl;
      u32 adc        = trk_response_fn(h);
      u32 strip_id   = strip_vid_hid_map_[h.volume];
      u8 channel     = static_cast<u8>(strip_id % 100);
      u32 remainder1 = strip_id / 100;
      u8 tmodule     = static_cast<u8>(remainder1 % 100);
      u32 remainder2 = remainder1 / 100;
      u8 row = static_cast<u8>(remainder2 % 10);
      u8 layer = static_cast<u8>(remainder2 / 10);
      TrkHit th;
      th.layer           = layer;
      th.row             = row;
      th.module          = tmodule;
      th.channel         = channel;
      th.adc             = adc;
      th.oscillator      = 0; // not simulated
      th.energy          = h.energy; 
      ev.trk_hits.push_back(th);
      //std::cout << ev.to_string() << std::endl;
    }
  }
  // debug, to study ttracker hits 
  //if (ev.trk_hits.size() == 0) {
  //  return None;
  //}
  bool ev_has_triggered = false;
  if (false) {
  //if (check_trigger) {
    if (g::GapsDigitizer::tof_trigger(r_hits)) {
      check_trigger = true;
      ev_has_triggered = true;
    } else {
      check_trigger = false;
    }
  } else {
    //check_trigger = false; 
    ev_has_triggered = true; // always trigger
  }
  // debug!! 
  ev_has_triggered = true;
  if (ev_has_triggered) {
    n_events++;
    return Some(ev);
  } else {
    return None;
  }
}

//----------------------------------------------------------

auto g::GapsDigitizer::load_tof_pulse_templates(const std::string& filename) -> void {
  // FIXME - we want to refactor this in a way that it returns Result and does 
  // not throw runtime errors
  g::TofPacketReader reader(filename);
  std::unordered_map<u8, g::PulseTemplatesBySide> loaded;
  usize count = 0;

  while (!reader.is_exhausted()) {
    auto result = reader.get_next_packet();
    if (!result.is_ok()) {
      if (reader.is_exhausted()) break; // Reader reports EOF this way.
      throw std::runtime_error("Failed reading TOF packets from " + filename);
    }
    auto packet = result.unwrap();
    if (packet.packet_type != PacketType::TofPulseTemplate) {
      continue;
    }

    usize offset = 0;
    auto pulse_res = g::TofPulseTemplate::from_bytestream(packet.payload, offset);
    if (!pulse_res.is_ok()) {
      throw std::runtime_error("IOError when reading pulse template from");
    }
    auto pulse = pulse_res.unwrap();
    // sanity checks
    if (offset != packet.payload.size()) {
      throw std::runtime_error("Unexpected trailing bytes in TOF template payload");
    }
    if (pulse.side > g::TofPulseTemplate::SIDE_B) {
      throw std::runtime_error("Invalid TOF template side");
    }
    loaded[pulse.paddle_id][pulse.side].push_back(std::move(pulse));
    ++count;
  }
  if (count == 0) {
    throw std::runtime_error("No TOF templates found in " + filename);
  }
  pulse_templates_.swap(loaded);
  n_pulse_templates_ = count;
  spdlog::info("Loaded {} TOF pulse templates from {}", count, filename);
}

//----------------------------------------------------------

auto g::GapsDigitizer::n_tof_pulse_templates() const -> usize {
  return n_pulse_templates_;
}

//----------------------------------------------------------

auto g::GapsDigitizer::tof_energy_to_peak_voltage(f64 energy_mev) noexcept -> f64 {
  return 35.26 * energy_mev / (1.0 + 0.021 * energy_mev);
}

//----------------------------------------------------------

auto g::GapsDigitizer::tof_paddle_attenuation_factor ( f64 position_mm, f64 paddle_length_mm,
                                                       u8 side ) noexcept -> f64 {
  const f64 x = position_mm - paddle_length_mm / 2.0;

  // for some reason, the two different sides have slightly 
  // different coefficients
  const f64 p0 = side == 0 ? 3.94     : 3.831;
  const f64 p1 = side == 0 ? 0.001211 : 0.0013103;
  const f64 p2 = side == 0 ? 20.87    : 23.42;

  const f64 x_side = side == 0 ? x : -x;
  const f64 numerator = std::exp(p0 - p1 * (x_side + paddle_length_mm / 2.0)) + p2;
  const f64 denominator = std::exp(p0 - p1 * paddle_length_mm / 2.0) + p2;
  return numerator / denominator;
}

//----------------------------------------------------------

// Exact half-open interval selection. Do not silently substitute templates.
auto g::GapsDigitizer::find_tof_pulse_template(
    u8 paddle_id, u8 side, f64 position_mm, f64 charge_pc) const
    -> const g::TofPulseTemplate* {
  if (side > g::TofPulseTemplate::SIDE_B ||
      !std::isfinite(position_mm) || !std::isfinite(charge_pc)) {
    return nullptr;
  }

  const auto paddle = pulse_templates_.find(paddle_id);
  if (paddle == pulse_templates_.end()) {
    return nullptr;
  }

  const auto& templates = paddle->second[side];
  const g::TofPulseTemplate* match = nullptr;
  for (const auto& pulse : templates) {
    if (position_mm >= pulse.position_low &&
        position_mm < pulse.position_high &&
        charge_pc >= pulse.charge_low &&
        charge_pc < pulse.charge_high) {
      // Overlapping templates would make selection ambiguous.
      if (match != nullptr) {
        throw std::runtime_error(
            "Overlapping TOF pulse template intervals for paddle " +
            std::to_string(static_cast<unsigned>(paddle_id)));
      }
      match = &pulse;
    }
  }
  return match;
}

//----------------------------------------------------------

auto g::GapsDigitizer::find_tof_pulse_template_by_peak(u8 paddle_id, u8 side, f64 position_mm, f64 paddle_length_mm,
                                                       f64 expected_peak_mv, bool saturated) -> const g::TofPulseTemplate* {
  if (side > 1 || !std::isfinite(expected_peak_mv)) {
    expected_peak_mv_not_finite++;
    return nullptr;
  }
  const auto paddle_it = pulse_templates_.find(paddle_id);
  if (paddle_it == pulse_templates_.end()) {
    //spdlog::warn("No pulse template found for paddle {}", paddle_id);
    no_template_for_pid++;
    return nullptr;
  }

  const auto& templates = paddle_it->second[side];

  // for saturated templates
  const f64 centered_position_mm = position_mm - paddle_length_mm / 2.0;
  const auto coeff = sat_tot_coeffs_for_position(centered_position_mm, paddle_length_mm);

  const g::TofPulseTemplate* best_template = nullptr;
  f64 best_metric = std::numeric_limits<f64>::infinity();
  f64 best_difference = std::numeric_limits<f64>::infinity();
  
  f64 template_peak_mv = 0;
  for (const auto& pulse : templates) {
     if (pulse.mean.empty() ||
        pulse.mean.size() != pulse.rms.size() ||
        !(pulse.time_step > 0.0)) {
      continue;
    }


    // Only consider templates from the correct position bin.
    if (position_mm < pulse.position_low ||
      position_mm >= pulse.position_high) {
      continue;
    }

    // Our measured templates are positive-going.
    template_peak_mv = *std::max_element(pulse.mean.begin(), pulse.mean.end());
    if (!std::isfinite(template_peak_mv)) {
      continue;
    }


    f64 metric = 0.0;
    
    if (saturated) {
      if (template_peak_mv < TOF_SATURATION_MV) {
        continue;
      }
      const auto time_ns = template_time_axis(pulse);
      const f64 tot = time_over_threshold(
          time_ns,
          pulse.mean,
          TOF_SAT_TOT_THRESHOLD_MV
      );

      if (tot < 0.0 || coeff.ws == 0.0) {
        continue;
      }

      const f64 slope = rise_slope_at_threshold(
          time_ns,
          pulse.mean,
          TOF_SAT_TOT_THRESHOLD_MV
      );

      const f64 target_tot =
          (expected_peak_mv -
           coeff.rsc -
           coeff.rss * slope) / coeff.ws;
      metric = std::abs(tot - target_tot);
    } else {
      metric = std::abs(
          expected_peak_mv - template_peak_mv
      );
    }

    if (std::isfinite(metric) && metric < best_metric) {
      best_metric = metric;
      best_template = &pulse;
    }
  }  
    //if (saturated && template_peak_mv < TOF_SATURATION_MV) {
    //  continue;
    //}
    //const f64 target_mv =  saturated ? TOF_SATURATION_MV : expected_peak_mv;
    //const f64 difference = std::abs(template_peak_mv - target_mv);
    //if (difference < best_difference) {
    //  best_difference = difference;
    //  best_template = &pulse;
    //}
  //}
  if (saturated && template_peak_mv < TOF_SATURATION_MV) {
    tof_hits_sat_no_templ_found++;
  }
  return best_template;
}

//----------------------------------------------------------

auto g::GapsDigitizer::scale_tof_pulse_template(const g::TofPulseTemplate& pulse,
                                                f64 expected_peak_mv) -> Vec<f64> {
  if (!std::isfinite(expected_peak_mv) || expected_peak_mv <= 0.0) {
    voltage_empty_expected_peak_invalid++;
    
    if (std::isinf(expected_peak_mv)) {
      volt_empty_exp_peak_nan++;
    }
    if (expected_peak_mv <= 0) {
      volt_empty_exp_peak_neg++;
    }
    if (std::isinf(expected_peak_mv)) { 
      volt_empty_exp_peak_inf++;
    }
    return {};
  }

  const auto peak_it = std::max_element(pulse.mean.begin(),pulse.mean.end());

  if (peak_it == pulse.mean.end() || !std::isfinite(*peak_it) || *peak_it <= 0.0) {
    voltage_empty_max_el_invalid++;
    return {};
  }

  const f64 scale = expected_peak_mv / *peak_it;

  std::vector<f64> waveform;
  waveform.reserve(pulse.mean.size());

  for (const auto voltage : pulse.mean) {
     waveform.push_back(
       std::clamp(scale * voltage, DRS4_MIN_MV, DRS4_MAX_MV)
     ); 
      
    //waveform.push_back(scale * voltage);
  }
  return waveform;
}

//----------------------------------------------------------
    
auto g::GapsDigitizer::tof_propagation_time_offset(f64 position_mm, f64 paddle_length_mm,
                                                   f64 velocity_mm_per_ns, u8 side) noexcept -> f64 {
  const f64 centered_position = position_mm - paddle_length_mm / 2.0;
  const f64 offset = centered_position / velocity_mm_per_ns;
  return side == 0 ? offset : -offset;
}

//----------------------------------------------------------

auto g::GapsDigitizer::add_waveform_fluctuations(
    Vec<f64>& waveform_mv,
    const Vec<f64>& rms_mv
) -> void {

  if (waveform_mv.size() != rms_mv.size()) {
    throw std::runtime_error(
        "Waveform and RMS vector sizes do not match"
    );
  }

  std::normal_distribution<f64> gaussian(0.0, 1.0);

  for (usize i = 0; i < waveform_mv.size(); ++i) {
    const f64 sigma_mv = 0.5 * std::abs(rms_mv[i]);

    waveform_mv[i] +=
        sigma_mv * gaussian(random_gen_);
  }
}
//auto g::GapsDigitizer::add_waveform_fluctuations(
//    Vec<f64>& waveform_mv,
//    const g::TofPulseTemplate& pulse,
//    f64 scale_factor
//) -> void {
//  std::normal_distribution<f64> gaussian(0.0, 1.0);
//  for (usize i = 0; i < waveform_mv.size(); ++i) {
//    const f64 sigma_mv = 0.5 * std::abs(scale_factor) * pulse.rms[i];
//    waveform_mv[i] += sigma_mv * gaussian(random_gen_);
//    // this is still our hobo approach to saturation
//    waveform_mv[i] = std::clamp(waveform_mv[i], DRS4_MIN_MV, DRS4_MAX_MV);
//  }
//}

//----------------------------------------------------------

auto g::GapsDigitizer::generate_tof_waveform(
    u8 paddle_id,
    u8 side,
    f64 energy_mev,
    f64 position_mm,
    f64 paddle_length_mm,
    f64 velocity_mm_per_ns,
    f64 hit_time_ns
) -> Option<g::SimTofWaveform> {

  const f64 expected_peak_mv =
      tof_energy_to_peak_voltage(energy_mev) *
      tof_paddle_attenuation_factor(
          position_mm,
          paddle_length_mm,
          side
      );

  const bool saturated = expected_peak_mv > TOF_SATURATION_MV;
  if (saturated) {
    tof_hits_sat++;
  }
  
  const auto* pulse = find_tof_pulse_template_by_peak(
      paddle_id,
      side,
      position_mm,
      paddle_length_mm,
      expected_peak_mv,
      saturated
  );

  if (pulse == nullptr) {
    return None;
  }

  if (pulse->mean.empty() ||
      pulse->mean.size() != pulse->rms.size()) {
    return None;
  }

  Vec<f64> voltage;
  Vec<f64> rms;

  if (saturated) {

    // Native saturated template, without amplitude scaling.
    voltage.assign(
        pulse->mean.begin(),
        pulse->mean.end()
    );

    rms.assign(
        pulse->rms.begin(),
        pulse->rms.end()
    );

    const f64 centered_position_mm =
        position_mm - paddle_length_mm / 2.0;

    const auto coeff = sat_tot_coeffs_for_position(
        centered_position_mm,
        paddle_length_mm
    );

    const auto time_ns = template_time_axis(*pulse);

    const f64 measured_tot = time_over_threshold(
        time_ns,
        voltage,
        TOF_SAT_TOT_THRESHOLD_MV
    );

    const f64 rise_slope = rise_slope_at_threshold(
        time_ns,
        voltage,
        TOF_SAT_TOT_THRESHOLD_MV
    );

    if (measured_tot >= 0.0 && coeff.ws != 0.0) {

      const f64 target_tot =
          (expected_peak_mv -
           coeff.rsc -
           coeff.rss * rise_slope) / coeff.ws;

      const f64 delta_width_ns =
          target_tot - measured_tot;

      adjust_saturated_pulse_width(
          voltage,
          rms,
          pulse->time_step,
          delta_width_ns,
          random_gen_
      );
    }

  } else {

    // Existing unsaturated amplitude scaling.
    voltage = scale_tof_pulse_template(
        *pulse,
        expected_peak_mv
    );

    if (voltage.empty()) {
      return None;
    }

    const f64 template_peak_mv = *std::max_element(
        pulse->mean.begin(),
        pulse->mean.end()
    );

    if (!(template_peak_mv > 0.0)) {
      return None;
    }

    const f64 scale_factor =
        expected_peak_mv / template_peak_mv;

    rms.reserve(pulse->rms.size());

    for (const auto value : pulse->rms) {
      rms.push_back(value * scale_factor);
    }
  }

  if (cfg.digi_tof_add_gauss_jitter) {
    add_waveform_fluctuations(voltage, rms);
  }

  const f64 propagation_offset_ns =
      tof_propagation_time_offset(
          position_mm,
          paddle_length_mm,
          velocity_mm_per_ns,
          side
      );

  auto wf = g::SimTofWaveform();

  wf.paddle_id = paddle_id;
  wf.voltage_mv = std::move(voltage);

  wf.time_start_ns =
      hit_time_ns +
      pulse->time_start +
      propagation_offset_ns;

  wf.time_step_ns = pulse->time_step;

  return Some(wf);
}

//auto g::GapsDigitizer::generate_tof_waveform(u8 paddle_id, u8 side, 
//                                             f64 energy_mev, f64 position_mm,
//                                             f64 paddle_length_mm, f64 velocity_mm_per_ns,
//                                             f64 hit_time_ns) -> Option<g::SimTofWaveform> {
//  const f64 expected_peak_mv =
//    tof_energy_to_peak_voltage(energy_mev)*tof_paddle_attenuation_factor(position_mm, paddle_length_mm, side);
// 
//  const bool saturated = expected_peak_mv > TOF_SATURATION_MV;
//  if (saturated) {
//    tof_hits_sat++;
//  }
//  const auto* pulse = find_tof_pulse_template_by_peak(paddle_id, side, position_mm, expected_peak_mv, saturated);
//  if (pulse == nullptr) {
//    wf_no_tof_pulse_found++;
//    return None;
//  }
//
//  Vec<f64> voltage;
//  f64 scale_factor = 1.0;
//  if (saturated) {
//    // Preserve the native saturated template.
//    voltage.assign(pulse->mean.begin(), pulse->mean.end());
//  } else { 
//    voltage = scale_tof_pulse_template(*pulse, expected_peak_mv);
//    if (voltage.empty()) {
//      wf_voltage_empty++;
//      if (energy_mev == 0.0) {
//        energy_mev_zero++;
//      }
//      return None;
//    }
//    const auto peak_it = std::max_element(pulse->mean.begin(), pulse->mean.end());
//    scale_factor = expected_peak_mv / *peak_it;
//  }
//  if (cfg.digi_tof_add_gauss_jitter) {
//    // this assumes the noise scales with the amplitude 
//    add_waveform_fluctuations(voltage, *pulse, scale_factor);
//  } 
//  const f64 propagation_offset_ns =  tof_propagation_time_offset( position_mm, paddle_length_mm,
//                                                                  velocity_mm_per_ns, side);
//  auto wf = g::SimTofWaveform(); 
//  wf.paddle_id  = paddle_id; 
//  wf.voltage_mv = std::move(voltage); 
//  wf.time_start_ns = hit_time_ns + pulse->time_start + propagation_offset_ns;
//  wf.time_step_ns  = pulse->time_step;
//  return Some(wf);
//}

//----------------------------------------------------------
   
auto g::GapsDigitizer::tof_response_fn_fallback(g::RecoHit& hit) -> void {
  const auto& paddle = paddles_.at(hit.volume);
  const f64 position_from_a = paddle.get_position_from_a(hit.x, hit.y, hit.z);
  const f64 position_centered = position_from_a - 10.0*paddle.length / 2.0;
  hit.energy = tof_energy_response_fn_simple( hit.energy, position_centered);
  hit.time = tof_timing_response_fn_simple(hit.time);
}

//----------------------------------------------------------

auto g::GapsDigitizer::digitize_tof_hit_fallback( const g::RecoHit& hit, f64 velocity_mm_per_ns) -> Option<g::TofHit> {
  auto reco = hit;
  // Apply the existing legacy response.
  tof_response_fn_fallback(reco);
  const auto& paddle = paddles_.at(hit.volume);
  const f64 paddle_length_mm = 10.0 * paddle.length;
  // Convert reconstructed energy back into peak voltage.
  const f64 energy = reco.energy;
  if (energy == 0) {
    tof_hits_digi_fallback_e0++;
  }
  const f64 peak_voltage_mv =
      35.26 * energy / (1.0 + 0.021 * energy);
  if (!std::isfinite(peak_voltage_mv)) {
      return None;
  }
  // Propagation time from paddle center to either end.
  const f64 center_to_end_ns =
      paddle_length_mm / (2.0 * velocity_mm_per_ns);

  g::TofHit tof_hit;
  tof_hit.paddle_id = paddle.paddle_id;
  tof_hit.version   = g::ProtocolVersion::V1;
  // First approximation: identical arrival times at both ends.
  tof_hit.time_a_f32 = static_cast<f32>(
      reco.time + center_to_end_ns
  );
  tof_hit.time_b_f32 = static_cast<f32>(
      reco.time + center_to_end_ns
  );
  // Equivalent peak voltages corresponding to legacy energy.
  tof_hit.peak_a_f32 = static_cast<f32>(peak_voltage_mv);
  tof_hit.peak_b_f32 = static_cast<f32>(peak_voltage_mv);
  // No waveform was generated, so these charges are unavailable.
  tof_hit.charge_a_f32 = 0.0f;
  tof_hit.charge_b_f32 = 0.0f;

  tof_hit.quality = g::HitQuality::Unknown;
  return Some(tof_hit);
}

//----------------------------------------------------------

auto g::GapsDigitizer::digitize_tof_hit(const RecoHit& hit, 
                                        f64 velocity_mm_per_ns)  -> Option<g::TofHit> {
  
  auto paddle = paddles_.at(hit.volume);
  f64  paddle_length_mm = 10*paddle.length; // FIXME - the cm/mm thing is getting really annoying! 
  f64  position_mm      = paddle.get_position_from_a(hit.x, hit.y, hit.z); // this is in mm
  //if (paddle_length_mm > 1700) {
  //  std::cout << "--- --- --- --- ---" << std::endl;
  //  std::cout << "POS " << position_mm << std::endl;
  //  std::cout << "LEN " << paddle_length_mm << std::endl;
  //  std::cout << "POS/LEN " << position_mm/ paddle_length_mm << std::endl;
  //}
  // Generate waveforms for both paddle ends.
  const auto waveform_a = generate_tof_waveform(
    paddle.paddle_id,
    0,
    hit.energy, // in MeV
    //energy_mev,
    position_mm,
    paddle_length_mm,
    velocity_mm_per_ns,
    hit.time
  );

  const auto waveform_b = generate_tof_waveform(
    paddle.paddle_id,
    1,
    hit.energy,
    position_mm,
    paddle_length_mm,
    velocity_mm_per_ns,
    hit.time
  );

  if (!waveform_a || !waveform_b) {
    n_failed_reason_wf_bad++;
    return None;
  }

  // Extract waveform features.
  const auto features_a = extract_tof_waveform_features(*waveform_a);
  const auto features_b = extract_tof_waveform_features(*waveform_b);

  if (features_a.is_err() || features_b.is_err()) {
    n_failed_reason_no_features++;
    return None;
  }

  const auto& a = features_a.try_ok();
  const auto& b = features_b.try_ok();

  // threshold for read out - this is different than the trigger but warrants a check
  const bool accepted = a.peak_voltage_mv > cfg.digi_tof_hit_threshold_mv && b.peak_voltage_mv > cfg.digi_tof_hit_threshold_mv;
  if (!accepted) {
    tof_hits_below_th++;
    return None;
  }
  // Our waveform timing is referenced to the paddle center.
  // Flight reconstruction expects propagation from the hit
  // to the paddle ends.
  const f64 center_to_end_ns = paddle_length_mm / (2.0 * velocity_mm_per_ns);
  
  g::TofHit tof_hit;
  tof_hit.paddle_id = paddle.paddle_id;
  tof_hit.version   = g::ProtocolVersion::V1;
  // Flight TofHit stores paddle length in cm.
  // we don't need the len
  //tof_hit.set_paddle_len(
  //  static_cast<f32>(paddle_length_mm / 10.0)
  //);

  tof_hit.time_a_f32 = static_cast<f32>(
    a.cfd_time_ns + center_to_end_ns
  );

  tof_hit.time_b_f32 = static_cast<f32>(
    b.cfd_time_ns + center_to_end_ns
  );

  tof_hit.peak_a_f32   = static_cast<f32>(a.peak_voltage_mv);
  tof_hit.peak_b_f32   = static_cast<f32>(b.peak_voltage_mv);
  tof_hit.charge_a_f32 = static_cast<f32>(a.charge_pc);
  tof_hit.charge_b_f32 = static_cast<f32>(b.charge_pc);
  tof_hit.quality      = g::HitQuality::Unknown;
  return tof_hit;
}
