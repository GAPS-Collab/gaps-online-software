/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include "result/result.h"
#include <format>
  
namespace gondola {

  /// A representation of a pair of waveforms from a single paddle
  struct RBWaveform {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
 
    /// The event id this waveform has been part of 
    u32       event_id     ;
    /// The internal RB identifier number for the RB which recorded 
    /// this waveform 
    u8        rb_id        ;
    /// The RB channel ID for the channel connected to the A-side  
    u8        rb_channel_a ; 
    /// The RB channel ID for the channel connected to the B-side  
    u8        rb_channel_b ;
    /// Trigger stop cell for DRS (Domino Ring Sampler)
    u16       stop_cell    ;
    /// Actual ADC values (uncalibrated) for connected paddle A-side 
    Vec<u16>  adc_a        ; 
    /// Actual ADC values (uncalibrated) for connected paddle B-side 
    Vec<u16>  adc_b        ;
    
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos) -> RBWaveform;
    auto to_string() const -> std::string;
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBWaveform& rh);
}
