#ifndef TOFHEADER_H_DEFINED
#define TOFHEADER_H_DEFINED

#include "tof_typedefs.h"

struct TofEventHeader {
  static const u16 HEAD   = 0xAAAA;
  static const u16 TAIL   = 0x5555;
  /// fixed size including head and tail
  static const usize SIZE = 47; 
  
  u32 run_id      ; 
  u32 event_id    ; 
  /// a reference to a timestamp
  /// which is not yet decided
  u32 timestamp32 ; 
  u16 timestamp16 ;  // -> 14 byres
  
  // reconstructed quantities
  u16 primary_beta        ; 
  u16 primary_beta_unc    ; 
  u16 primary_charge      ; 
  u16 primary_charge_unc  ; 
  u16 primary_outer_tof_x ; 
  u16 primary_outer_tof_y ; 
  u16 primary_outer_tof_z ; 
  u16 primary_inner_tof_x ; 
  u16 primary_inner_tof_y ; 
  u16 primary_inner_tof_z ; //-> 20bytes primary 

  u8 nhit_outer_tof       ;  
  // no need to save this, can be 
  // rereated from paddle_info.size() - nhit_outer_tof
  u8 nhit_inner_tof       ;

  u8 trigger_info         ; 
  u8 ctr_etx              ;

  // this field can be debated
  // the reason we have it is 
  // that for de/serialization, 
  // we need to know the length 
  // of the expected bytestream.
  u8 n_paddles            ; // we don't have more than 
                               // 256 paddles.

  /// String representation for printing to output
  std::string to_string() const;

  /// get the timestamp
  f64 get_timestamp48() const;

  static TofEventHeader from_bytestream(const Vec<u8> &stream,
                                        u64 &pos);

}; // end TofEventHeader

std::ostream& operator<<(std::ostream& os, const TofEventHeader& h);

#endif
