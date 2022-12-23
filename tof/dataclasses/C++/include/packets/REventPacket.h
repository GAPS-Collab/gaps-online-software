#ifndef REVENTPACKET_H_INCLUDED
#define REVENTPACKET_H_INCLUDED

#include <vector>

#include "RPaddlePacket.h"

// no JOKE!
#define REVENTPACKETSIZEFIXED 42 
#define REVENTPACKETVERSION "rev1.0"

/**
 * Version 1.0
 * -> extends head and tail to 2 bytes
 */
struct REventPacket {
  unsigned short head = 0xAAAA;

  unsigned short p_length;
  uint32_t event_ctr;
  uint64_t utc_timestamp;

  // reconstructed quantities
  uint16_t primary_beta;
  uint16_t primary_beta_unc;
  uint16_t primary_charge;
  uint16_t primary_charge_unc;
  uint16_t primary_outer_tof_x;
  uint16_t primary_outer_tof_y;
  uint16_t primary_outer_tof_z;
  uint16_t primary_inner_tof_x;
  uint16_t primary_inner_tof_y;
  uint16_t primary_inner_tof_z;

  unsigned char nhit_outer_tof;
  // no need to save this, can be 
  // rereated from paddle_info.size() - nhit_outer_tof
  unsigned char nhit_inner_tof;

  unsigned char trigger_info;
  unsigned char ctr_etx;

  unsigned short tail = 0x5555;

  
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
  std::vector<unsigned char>serialize() const;

  /**
   * Transcode from bytestream
   *
   * Returns:
   *    position where the event is found in the bytestream
   *    (tail position +=1, so that bytestream can be iterated
   *    over easily)
   */
  unsigned int deserialize(std::vector<unsigned char>& payload,
                           unsigned int start_pos=0);


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
