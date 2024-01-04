#ifndef MASTERTRIGGERPACKET_H_INCLUDED
#define MASTERTRIGGERPACKET_H_INCLUDED

/**************************************
 *
 * MasterTrigger event format
 * Feb 2023
 * 
 ***********************/

#include "tof_typedefs.h"
static const usize N_LTBS = 20;
static const usize N_CHN_PER_LTB = 16;

struct MasterTriggerPacket {
  static const u64 SIZE = 45;
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  u32 event_id        ; 
  u32 timestamp       ; 
  u32 tiu_timestamp   ; 
  u32 gps_timestamp_32; 
  u32 gps_timestamp_16; 
  u32 board_mask      ;
  u8  n_paddles       ;
  Vec<u32> hits       ;
  u32 crc             ;
 
  /**
   * String representation
   *
   */
  std::string to_string() const;

  /**
   * Reset all fields to 0 values
   * FIXME - nan would be better
   */
  void reset();

  //! Decode packet from byte representation
  static MasterTriggerPacket from_bytestream(Vec<u8> &payload, 
                                             u64 &pos);


  //! The hit board ids
  Vec<u8> get_hit_board_ids() const;

  //! The hit board ids 
  Vec<u8> get_hit_paddle_ids() const;

};


#endif
