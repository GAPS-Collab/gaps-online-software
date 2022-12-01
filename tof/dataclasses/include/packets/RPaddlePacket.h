#ifndef RPADDLEPACKET_H_INCLUDED
#define RPADDLEPACKET_H_INCLUDED

#include <cstdint>
#include <vector>
#include <string>

#define RPADDLEPACKETSIZE 26
#define RPADDLEPACKETVERSION "rev1.0"



struct RPaddlePacket  {

  unsigned short head = 0xF0F0;

  unsigned short p_length= RPADDLEPACKETSIZE;
  //uint32_t event_ctr;
  //unsigned char utc_timestamp[8];

  unsigned char paddle_id;
  unsigned short time_a;
  unsigned short time_b;
  unsigned short peak_a;
  unsigned short peak_b;
  unsigned short charge_a;
  unsigned short charge_b;
  unsigned short charge_min_i;
  unsigned short x_pos;
  unsigned short t_average;

  unsigned char ctr_etx;
  unsigned short tail = 0xF0F; 

  // convert the truncated values
  unsigned short get_paddle_id() const;
  float get_time_a()             const;
  float get_time_b()             const;
  float get_peak_a()             const;
  float get_peak_b()             const;
  float get_charge_a()     const;
  float get_charge_b()     const;
  float get_charge_min_i() const;
  float get_x_pos()        const;
  float get_t_avg()        const;
  // setters
  void set_time_a(double);
  void set_time_b(double);
  void set_peak_a(double);
  void set_peak_b(double);
  void set_charge_a(double);
  void set_charge_b(double);
  void set_charge_min_i(double);
  void set_x_pos(double);
  void set_t_avg(double);

  // don't serialize (?)
  std::string version = RPADDLEPACKETVERSION; // packet version


  // PaddlePacket legth is fixed
  static unsigned short calculate_length();
  void reset();

  std::vector<unsigned char> serialize() const; 
  unsigned int deserialize(std::vector<unsigned char>& bytestream, 
                                unsigned int start_pos);
 
  // easier print out
  std::string to_string() const;
};

std::ostream& operator<<(std::ostream& os, const RPaddlePacket& pad);

#endif
