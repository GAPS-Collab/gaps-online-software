#pragma once 

#include <random>

#include "result/result.h"

#include "gondola_typedefs.hpp"
#include "tracklet.hpp"
#include "events/mc_event.hpp"
#include "events/telemetry_event.hpp"

#include "database/tof_paddle.hpp"
#include "database/tracker_transfer_fn.hpp"
#include "tof_pulse_template.hpp"

#include "sim_config.hpp"

// investigate why this is not needed, 
// we should keep the headers clean of 
// using namespace
//namespace r = result;

namespace gondola {

  struct SimTofWaveform {
    Vec<f64> voltage_mv;
    f64 time_start_ns;
    f64 time_step_ns;
    u8  paddle_id;

    [[nodiscard]]
    auto time_at(std::size_t index) const noexcept -> f64 {
      return time_start_ns + static_cast<f64>(index) * time_step_ns;
    }
  };

  struct SimTofWaveformPeak {
    f64 voltage_mv;
    f64 time_ns;
    usize sample_index;
  };

  struct SimTofWaveformFeatures {
    f64 peak_voltage_mv;
    f64 peak_time_ns;
    f64 cfd_time_ns;
    f64 integral_mv_ns;
    f64 charge_pc;
  };


  [[nodiscard]]
  auto find_tof_waveform_peak(const SimTofWaveform& waveform) -> Option<SimTofWaveformPeak>;
 
  [[nodiscard]]
  auto integrate_tof_waveform(const SimTofWaveform& waveform, f64 impedance_ohm = 50.0) -> f64;

  auto extract_tof_waveform_features(const SimTofWaveform& waveform, f64 cfd_fraction = 0.25)
    -> r::Result<SimTofWaveformFeatures, AnalysisError>;

  struct GapsDigitizer {
 
    GapsDigitizer(const SimConfig& cfg);
    ~GapsDigitizer();
    
    /// merge hits together if they are only seperated by min_time_delta 
    /// in time and are in the same volume 
    ///
    /// @param min_time_delta : default value means to merge all hits in the 
    ///                         same volume
    static auto merge_hits (const Vec<RecoHit>& hits, f64 min_time_delta = 1e20) -> Vec<RecoHit>;

    /// merge hits together if they are only sepearated by min_time_delta and 
    /// are in the same volume    
    static auto integrate_mc_hits(const Vec<McHit>& hits, f64 min_time_delta) -> Vec<RecoHit>;
   
    //auto tof_response_fn_simple(RecoHit& hit)                            -> void;
    auto tof_energy_response_fn_simple(f64 energy, f64 pos_along_paddle) -> f64;
    auto tof_timing_response_fn_simple(f64 time)                         -> f64;
    /// smear the position along the paddle and returh its value for the 
    /// energy calculation. 
    /// This will modify the hit
    //auto tof_pos_response_fn_simple(RecoHit& hit)                        -> f64; 
    /// Re-calculate energy for the hit and additionally return calculated adc values 
    auto trk_response_fn(RecoHit& hit)                                   -> u32;
    auto digitize_event(const McEvent& ev, bool& check_trigger)          -> Option<TelemetryEvent>;
    
    // From here on, this is related to TOF digitization 
    //
    //
    auto tof_trigger(const Vec<RecoHit>& hits)                     const -> bool;
    auto load_tof_pulse_templates(const std::string& filename)           -> void;
    auto n_tof_pulse_templates() const                                   -> usize;
    
    [[nodiscard("Check whether a matching template exists")]]
    auto find_tof_pulse_template(u8 paddle_id, u8 side,
                                 f64 position_mm, f64 charge_pc)   const -> const TofPulseTemplate*;
    
    [[nodiscard("Returns the best matching template, or nullptr")]]
    auto find_tof_pulse_template_by_peak(u8 paddle_id, u8 side, f64 position_mm, f64 paddle_length_mm,
                                         f64 expected_peak_mv, bool saturated = false)  -> const TofPulseTemplate*;
    
    [[nodiscard("Returns the expected peak voltage in mV")]]
    static auto tof_energy_to_peak_voltage(f64 energy_mev) noexcept -> f64;
    
    [[nodiscard("Returns the position-dependent attenuation factor")]]
    static auto tof_paddle_attenuation_factor(f64 position_mm, f64 paddle_length_mm,
                                              u8 side)          noexcept -> f64;
    

    [[nodiscard("Returns the amplitude-scaled template waveform")]]
    auto scale_tof_pulse_template(const TofPulseTemplate& pulse,
                                  f64 expected_peak_mv) -> Vec<f64>;

    [[nodiscard("Returns the relative propagation delay in ns")]]
    static auto tof_propagation_time_offset(f64 position_mm, f64 paddle_length_mm,
                                            f64 velocity_mm_per_ns, u8 side) noexcept -> f64; 

    [[nodiscard("Returns a generated waveform, if possible")]]
    auto generate_tof_waveform(u8 paddle_id, u8 side, f64 energy_mev, f64 position_mm,
                               f64 paddle_length_mm, f64 velocity_mm_per_ns, f64 hit_time_ns
                              )  -> Option<SimTofWaveform>;

    //[[nodiscard]]
    //auto digitize_tof_hit(u8 paddle_id, f64 energy_mev,
    //                      f64 position_mm, f64 paddle_length_mm,
    //                      f64 velocity_mm_per_ns, f64 hit_time_ns) const -> Option<gondola::TofHit>;
    [[nodiscard]]
    auto digitize_tof_hit(const RecoHit& hit, f64 velocity_mm_per_ns = 1.54e8) -> Option<gondola::TofHit>;
    // Speed of the pulse, approximately 55% of the speed of light.
    
    auto tof_response_fn_fallback(RecoHit& hit) -> void;
    auto digitize_tof_hit_fallback( const RecoHit& hit, f64 velocity_mm_per_ns = 1.54e8) -> Option<gondola::TofHit>;

    /// Print error/debugging statistics, counter etc. to std::out
    auto print_statistics() -> void;

    auto to_string() const -> std::string;
    
    bool use_digitizer             = false;
    bool tof_add_gauss_jitter      = false;
    
    // keep a copy of the config 
    SimConfig cfg;
    usize n_events                    = 0;
    // debugging counters for the TOF part

    /// The total number of TOF RecoHits we tried to put through 
    /// the machinery 
    usize tof_hits_attempted          = 0;
    /// The number of TOF Reco hits we could successfully convert 
    /// using the machinery  
    usize tof_hits_succeeded          = 0;
    usize tof_hits_failed             = 0;
    /// The number of TOF hits which, after reconstruction fall below 
    /// the "HIT" threshold
    usize tof_hits_below_th           = 0;
    /// The number of TOF hits we deliberately excluded from the events 
    /// by the config file 
    usize tof_hits_excluded_rb_mask   = 0;
    /// The number of TOF hits which are saturated, but we don't find a 
    /// saturated pulse template
    usize tof_hits_sat_no_templ_found = 0;
    /// The total number of saturated TOF hits
    usize tof_hits_sat                = 0;
    /// The number of (Tof) hits we created by using the fallback 
    /// method 
    usize tof_hits_digi_fallback      = 0;
    /// The number of (Tof) hits we created by using the fallback 
    /// method which then has a resulting energy of zero
    usize tof_hits_digi_fallback_e0   = 0;

    usize n_failed_reason_wf_bad      = 0;
    usize n_failed_reason_no_features = 0;
    usize wf_no_tof_pulse_found       = 0;
    usize wf_voltage_empty            = 0;
    usize expected_peak_mv_not_finite = 0;
    usize no_template_for_pid         = 0;
    usize voltage_empty_expected_peak_invalid = 0;
    usize voltage_empty_max_el_invalid        = 0;
    usize volt_empty_exp_peak_nan     = 0;
    usize volt_empty_exp_peak_neg     = 0;
    usize volt_empty_exp_peak_inf     = 0;
    usize energy_mev_zero             = 0;
    usize zero_energy_hits            = 0;
    usize n_total_mc_hits             = 0;
    usize n_total_mc_hits_zero_en     = 0;
  private:

    //auto add_waveform_fluctuations(Vec<f64>& waveform_mv,
    //                               const TofPulseTemplate& pulse,
    //                               f64 scale_factor) -> void;
    auto add_waveform_fluctuations(
        Vec<f64>& waveform_mv,
        const Vec<f64>& rms_mv) -> void;

    //std::random_device  gauss_rng_;
    std::mt19937                  random_gen_;
    std::normal_distribution<f64> tof_volt_smearing_;
    std::normal_distribution<f64> tof_time_smearing_;
    std::normal_distribution<f64> tof_pos_smearing_;
    std::normal_distribution<f64> trk_energy_smearing_;
    /// A map of all TOF paddles, mapping is by volumeID -> TofPaddle
    TofPaddleMapVolId             paddles_;
    /// Tof Pulse templates as stored in an external file
    std::unordered_map<u8, PulseTemplatesBySide> pulse_templates_;
    // Tracker strips by volume id
    TrkStripMap                   strips_;
    TrkStripTransferFnMap         strip_tfns_; 
    TrkStripPedMap                strip_pedestals_;
    HashMap<u32,bool>             strip_mask_; 
    HashMap<u32,u32>              strip_vid_hid_map_; 

    bool trk_add_cmn_noise_   = false;
    usize n_pulse_templates_  = 0;
  };
}
