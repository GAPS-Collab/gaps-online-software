#ifndef REVENTPACKET_H_INCLUDED
#define REVENTPACKET_H_INCLUDED

#include <vector>

#include "tof_typedefs.h"
#include "RPaddlePacket.h"


// no JOKE!
#define REVENTPACKETSIZEFIXED 42 
#define REVENTPACKETVERSION "rev1.1"

/**
 * Version 1.0
 * -> extends head and tail to 2 bytes
 *
 * Version 1.1
 * -> adds n_paddles (char) field. This 
 *    is used to calculate the total length
 *    of the packet. We use the p_length field
 *    for that and just rename it. 
 *
 * Version 1.2
 * -> removes 64bit timestamp and replaces 
 *    it with 32bit  + 16bit. 
 *    Since the timestamps is only 48 bit 
 *    anyway.
 */
struct REventPacket {
  u16 head = 0xAAAA;

  u32 event_ctr;
  u32 timestamp_32;
  u16 timestamp_16;
  u16 n_paddles;


  // reconstructed quantities
  u16 primary_beta;
  u16 primary_beta_unc;
  u16 primary_charge;
  u16 primary_charge_unc;
  u16 primary_outer_tof_x;
  u16 primary_outer_tof_y;
  u16 primary_outer_tof_z;
  u16 primary_inner_tof_x;
  u16 primary_inner_tof_y;
  u16 primary_inner_tof_z;

  u8 nhit_outer_tof;
  // no need to save this, can be 
  // rereated from paddle_info.size() - nhit_outer_tof
  u8 nhit_inner_tof;

  u8 trigger_info;
  u8 ctr_etx;

  u16 tail = 0x5555;

  
  // payload
  std::vector<RPaddlePacket> paddle_info;

  /**
   * String representation
   *
   */
  std::string to_string(bool summarize_paddle_packets=false) const;

  /**
   * Reset all fields to 0 values
   * FIXME - nan would be better
   */
  void reset();

  /**
   * Add a paddle info packet to the event
   *
   */
  void add_paddle_packet(RPaddlePacket const &pkt);

  /**
   * Calculate the sise of the packet in bytes
   *
   */
  unsigned short calculate_length() const;

  /**
   * Transcode to bytestream
   *
   *
   */
  vec_u8 serialize() const;

  /**
   * Transcode from bytestream
   *
   * Returns:
   *    position where the event is found in the bytestream
   *    (tail position +=1, so that bytestream can be iterated
   *    over easily)
   */
  unsigned int deserialize(vec_u8 &payload,
                           u64 start_pos=0);


  /**
   * Check if paddle is broken after de/serialization
   *
   */
  bool is_broken();
 
  private:
    // do not serialize
    unsigned short p_length_fixed = REVENTPACKETSIZEFIXED;
    bool broken = false; // mark broken package
}; // end REventPacket



std::ostream& operator<<(std::ostream& os, const REventPacket& h);



#endif
