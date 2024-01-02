#ifndef RPADDLEPACKET_H_INCLUDED
#define RPADDLEPACKET_H_INCLUDED

#include <cstdint>
#include <vector>
#include <string>

#include "tof_typedefs.h"

// adds 6 bytes for timestamps
#define RPADDLEPACKETSIZE 30
#define RPADDLEPACKETVERSION "1.2"

/***********************************************************
 * The "reduced paddle packet" holds analyzed waveform 
 * information. 
 *
 **********************************************************/

[[deprecated("RPaddlePacket is deprecated in favor of TofHit (since 0.8.3)")]]
struct RPaddlePacket  {

  u16 head = 0xF0F0;

  u8 paddle_id;
  u16 time_a;
  u16 time_b;
  u16 peak_a;
  u16 peak_b;
  u16 charge_a;
  u16 charge_b;
  u16 charge_min_i;
  u16 x_pos;
  u16 t_average;

  u32 timestamp_32;
  u16 timestamp_16;


  u8 ctr_etx;
  u16 tail = 0xF0F; 

  // convert the truncated values
  u16 get_paddle_id()    const;
  f32 get_time_a()       const;
  f32 get_time_b()       const;
  f32 get_peak_a()       const;
  f32 get_peak_b()       const;
  f32 get_charge_a()     const;
  f32 get_charge_b()     const;
  f32 get_charge_min_i() const;
  f32 get_x_pos()        const;
  f32 get_t_avg()        const;
  // setters
  void set_time_a      (f64);
  void set_time_b      (f64);
  void set_peak_a      (f64);
  void set_peak_b      (f64);
  void set_charge_a    (f64);
  void set_charge_b    (f64);
  void set_charge_min_i(f64);
  void set_x_pos       (f64);
  void set_t_avg       (f64);

  // don't serialize (?)
  std::string version = RPADDLEPACKETVERSION; // packet version



  //! If the paddle is broken, it is utter trash. Most likely, nothing can be salvaged.
  bool is_broken();

  // PaddlePacket legth is fixed
  static u16 calculate_length();
  void reset();

  Vec<u8> serialize() const; 
  static RPaddlePacket from_bytestream(const Vec<u8> &bytestream, 
                                       u64 &pos);
 
  // easier print out
  std::string to_string() const;
  
  private:
    // don't serialize
    bool broken;
};

std::ostream& operator<<(std::ostream& os, const RPaddlePacket& pad);

#endif
