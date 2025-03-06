#ifndef TOFHEADER_H_DEFINED
#define TOFHEADER_H_DEFINED

#include "result/result.h"

#include "errors.hpp"
#include "tof_typedefs.h"

namespace r = result;
namespace g = Gaps;

struct TofEventHeader {
  static const u16 HEAD   = 0xAAAA;
  static const u16 TAIL   = 0x5555;
  /// fixed size including head and tail
  static const usize SIZE = 47; 
  
  u32 run_id      = 0; 
  u32 event_id    = 0; 
  /// a reference to a timestamp
  /// which is not yet decided
  u32 timestamp32 = 0; 
  u16 timestamp16 = 0;  // -> 14 byres
  
  // reconstructed quantities
  u16 primary_beta        = 0; 
  u16 primary_beta_unc    = 0; 
  u16 primary_charge      = 0; 
  u16 primary_charge_unc  = 0; 
  u16 primary_outer_tof_x = 0; 
  u16 primary_outer_tof_y = 0; 
  u16 primary_outer_tof_z = 0; 
  u16 primary_inner_tof_x = 0; 
  u16 primary_inner_tof_y = 0; 
  u16 primary_inner_tof_z = 0; //-> 20bytes primary 

  u8 nhit_outer_tof       = 0;  
  // no need to save this, can be 
  // rereated from paddle_info.size() - nhit_outer_tof
  u8 nhit_inner_tof       = 0;

  u8 trigger_info         = 0; 
  u8 ctr_etx              = 0;

  // this field can be debated
  // the reason we have it is 
  // that for de/serialization, 
  // we need to know the length 
  // of the expected bytestream.
  u8 n_paddles            = 0; // we don't have more than 
                               // 256 paddles.

  /// String representation for printing to output
  std::string to_string() const;

  /// get the timestamp
  f64 get_timestamp48() const;

  static auto from_bytestream(const Vec<u8> &stream, u64 &pos)
    -> r::Result<TofEventHeader, g::IOError>;

}; // end TofEventHeader

std::ostream& operator<<(std::ostream& os, const TofEventHeader& h);

#endif
