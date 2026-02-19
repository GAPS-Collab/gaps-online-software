/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include "result/result.h"

#include "gondola.hpp"
#include "errors.hpp"
#include "database.h"
#include "version.h" 

namespace r = result;

namespace gondola {

  ///Reconstructed waveform peak information
  ///
  ///There should be one TofHit per reconstructed
  ///peak
  struct TofHit  {
    static constexpr u16 HEAD = 0xF0F0;
    static constexpr u16 TAIL = 0xF0F;
  
    u8   paddle_id;
  
    // new variables for V1
    ProtocolVersion version;
    f32 baseline_a;
    f32 baseline_a_rms;
    f32 baseline_b;
    f32 baseline_b_rms;
    f32 phase;
  
    // event wide calculated time
    f32 event_t0     = 0;
  
    u32 timestamp32;
    u16 timestamp16;
    

    // don't serialize
    f32 paddle_len    = 0;  
    f32 coax_cbl_time = 0;
    f32 hart_cbl_time = 0;
 
    u8 ctr_etx;
    u16 tail = 0xF0F; 
  
    auto get_time_a()       const -> f32;
    auto get_time_b()       const -> f32;
    auto get_peak_a()       const -> f32;
    auto get_peak_b()       const -> f32;
    auto get_charge_a()     const -> f32;
    auto get_charge_b()     const -> f32;
    auto get_x_pos()        const -> f32;
    /// If the two reconstructed pulse times are not related to each other by the paddle length,
    /// meaning that they can't be caused by the same event, we dub this hit as "not following
    /// causality"
    auto obeys_causality()  const -> bool; 
    /// get the interaction time of the particle,
    /// not accounting for cable len and global phase
    auto get_t0_relative()  const -> f32;
    auto get_timestamp48()  const -> f64;
    
    /// time-over-threshold for paddle end A for the 
    /// lower threshold (see config file for value)
    auto get_tot_low_a()    const -> f32;
    /// time-over-threshold for paddle end B for the 
    /// lower threshold (see config file for value)
    auto get_tot_low_b()    const -> f32;
    /// time-over-threshold for paddle end A for the 
    /// higher threshold (see config file for value)
    auto get_tot_high_a()    const -> f32;
    /// time-over-threshold for paddle end B for the 
    /// higher threshold (see config file for value)
    auto get_tot_high_b()     const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the lower threshold and the 
    /// waveform for side A
    auto get_tot_slp_low_a()  const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the lower threshold and the 
    /// waveform for side B
    auto get_tot_slp_low_b()  const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the higher threshold and the 
    /// waveform for side A
    auto get_tot_slp_high_a() const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the higher threshold and the 
    /// waveform for side B
    auto get_tot_slp_high_b() const -> f32;


    /// The paddle length will not be in the packet,
    /// but has to be added after the fact
    void set_paddle_len(f32 paddle_len);
  
    #if BUILD_CXX_DB
    auto set_paddle(const TofPaddle& paddle) -> void;
    auto get_phase_delay() const -> f32;
    auto get_cable_delay() const -> f32;
    auto get_t0()          const -> f32;
    auto get_edep()        const -> f32;
    #endif
  
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> r::Result<TofHit,IOError>;
   
    // String representation for printing
    auto to_string() const -> std::string;
    
    private:
      f32 time_a_f32   = 0;
      f32 time_b_f32   = 0;
      f32 peak_a_f32   = 0;
      f32 peak_b_f32   = 0;
      f32 charge_a_f32 = 0;
      f32 charge_b_f32 = 0;

      // new (2025/26) variables to deal with 
      // pulse saturation.
      // These are variables for time-over-threshold
      f32 tot_low_a      = 0;
      f32 tot_low_b      = 0;
      f32 tot_high_a     = 0;
      f32 tot_high_b     = 0;
      f32 tot_slp_low_a  = 0;
      f32 tot_slp_low_b  = 0;
      f32 tot_slp_high_a = 0;
      f32 tot_slp_high_b = 0;
  };
  
  std::ostream& operator<<(std::ostream& os, const TofHit& pad);
}
