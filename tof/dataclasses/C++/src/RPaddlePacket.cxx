#include "packets/RPaddlePacket.h"
#include "parsers.h"
#include <spdlog/spdlog.h>

void RPaddlePacket::reset()
{
  head = 0xF0F0; 

  paddle_id    = 0x00;
  time_a       = 0;
  time_b       = 0;
  peak_a       = 0;
  peak_b       = 0;
  charge_a     = 0;
  charge_b     = 0;
  charge_min_i = 0;
  x_pos        = 0;
  t_average    = 0;
  /// also different tail here, 
  //  so we can find the packet in 
  //  the REventstream
  ctr_etx = 0x00;

  //  timestamps for debugging - they 
  //  most likely will go away   
  timestamp_32  = 0;
  timestamp_16  = 0;

  tail = 0xF0F;
  broken = false;
}

/*******************************************/

unsigned short RPaddlePacket::calculate_length()
{
  // currently the lenght of the paddle packet is fixed
  // FIXME
  return RPADDLEPACKETSIZE; //previously 38
}

/*******************************************/

// getters
u16 RPaddlePacket::get_paddle_id() const
{
   return (unsigned short)paddle_id;
}

f32 RPaddlePacket::get_time_a() const
{
   f32 prec = 0.004;
   return prec*time_a;
}

f32 RPaddlePacket::get_time_b() const
{
  f32 prec = 0.004;//ns
  return prec*time_b;
}

f32 RPaddlePacket::get_peak_a() const
{
  f32 prec = 0.2;
  return prec*peak_a;
}

f32 RPaddlePacket::get_peak_b() const
{
  f32 prec = 0.2;
  return prec*peak_b;
}

f32 RPaddlePacket::get_charge_a() const
{
  f32 prec = 0.01; //pC
  return prec*charge_a - 50;
}

f32 RPaddlePacket::get_charge_b() const
{
  f32 prec = 0.01;
  return prec*charge_b - 50;
}

f32 RPaddlePacket::get_charge_min_i() const
{
  f32 prec = 0.002;// minI
  return prec*charge_min_i - 10;
}

f32 RPaddlePacket::get_x_pos() const
{
  // FIXME - check if it is really in the middle
  f32 prec = 0.005; //cm
  return prec*x_pos - 163.8;
}

f32 RPaddlePacket::get_t_avg() const
{
  f32 prec = 0.004;//ps
  return prec*t_average;
}


/*******************************************/

Vec<u8> RPaddlePacket::serialize() const
{
  Vec<u8> buffer(RPADDLEPACKETSIZE);
  usize pos = 0; // position in bytestream

  Gaps::u16_to_le_bytes(head, buffer, pos);
  buffer[pos] = paddle_id; pos+=1;

  //encode_ushort(paddle_id, buffer, pos); pos+=2;
  Gaps::u16_to_le_bytes(time_a, buffer, pos); 
  Gaps::u16_to_le_bytes(time_b, buffer, pos); 
  Gaps::u16_to_le_bytes(peak_a, buffer, pos); 
  Gaps::u16_to_le_bytes(peak_b, buffer, pos); 
  Gaps::u16_to_le_bytes(charge_a, buffer, pos); 
  Gaps::u16_to_le_bytes(charge_b, buffer, pos); 
  Gaps::u16_to_le_bytes(charge_min_i, buffer, pos); 
  Gaps::u16_to_le_bytes(x_pos, buffer, pos); 
  Gaps::u16_to_le_bytes(t_average, buffer, pos); 

  buffer[pos] = ctr_etx;        pos+=1;

  Gaps::u32_to_le_bytes(timestamp_32, buffer, pos);
  Gaps::u16_to_le_bytes(timestamp_16, buffer, pos);
  Gaps::u16_to_le_bytes(tail, buffer, pos);  // done
  return buffer; 
}

/*******************************************/

u32 RPaddlePacket::deserialize(Vec<u8> &bytestream,
                               u32 start_pos) {
 reset();
     	// start from position in bytestream
 //u16 value; 
 //u32 end_pos = start_pos;

 //// find start marker in bytestream
 //for (size_t k=start_pos;k<bytestream.size();k++)
 //{
 //  value = decode_ushort(bytestream, start_pos=k);
 //  if (head == value)
 //   {
 //    // end pos should point to the start
 //    // of the next new value
 //    end_pos = k+2;
 //    break;
 //   }
 //}

 //u64 pos = end_pos; // position in bytestream
 //u16 expected_packet_size = Gaps::u16_from_le_bytes(bytestream, pos);

 //event_ctr = decode_uint32(bytestream, pos); pos+=4;
 u64 pos = start_pos;
 u16 maybe_header = Gaps::u16_from_le_bytes(bytestream, pos);
 if (maybe_header != head) {
   spdlog::error("Can not find HEADER at presumed position. Maybe give a different value for start_pos?");
 }
 paddle_id     = bytestream[pos]; pos+=1;
 time_a        = Gaps::u16_from_le_bytes(bytestream, pos); 
 time_b        = Gaps::u16_from_le_bytes(bytestream, pos); 
 //std::cout << " " << time_a << " " << time_b << " " << charge_a << " " << charge_b << std::endl;
 peak_a        = Gaps::u16_from_le_bytes(bytestream, pos); 
 peak_b        = Gaps::u16_from_le_bytes(bytestream, pos); 
 charge_a      = Gaps::u16_from_le_bytes(bytestream, pos); 
 charge_b      = Gaps::u16_from_le_bytes(bytestream, pos); 
 charge_min_i  = Gaps::u16_from_le_bytes(bytestream, pos); 
 x_pos         = Gaps::u16_from_le_bytes(bytestream, pos); 
 t_average     = Gaps::u16_from_le_bytes(bytestream, pos); 

 ctr_etx = bytestream[pos]; pos+=1;

 timestamp_32 = Gaps::u32_from_le_bytes(bytestream, pos);
 timestamp_16 = Gaps::u16_from_le_bytes(bytestream, pos);

 // FIXME checks - packetlength, checksum ?
 tail = Gaps::u32_from_le_bytes(bytestream, pos);
 if (tail != 0xF0F) {
   broken = true;
 }
 return pos; 
}

/*******************************************/

std::string RPaddlePacket::to_string() const
{
  std::string repr = "";
  repr += "RPADDLEPACKET-----------------------\n";
  repr += "HEAD "          + std::to_string(head              ) + "\n";
  repr += "-- BROKEN "     + std::to_string(broken            ) + "\n";
  //repr += "EVENT CTR "     + std::to_string(event_ctr         ) + "\n";
  //repr += "UTC TS "        + std::to_string(utc_timestamp     ) + "\n";
  repr += "PADDLE ID "     + std::to_string(get_paddle_id()   ) + "\n";
  repr += "TIMESTAMP 32 "  + std::to_string(timestamp_32      ) + "\n";
  repr += "TIMESTAMP 16 "  + std::to_string(timestamp_16      ) + "\n";
  repr += "PTIME_A "       + std::to_string(get_time_a()      ) + "\n";
  repr += "PTIME_B "       + std::to_string(get_time_b()      ) + "\n";
  repr += "PEAK_A "        + std::to_string(get_peak_a()      ) + "\n";
  repr += "PEAK_B "        + std::to_string(get_peak_b()      ) + "\n";
  repr += "CHARGE_A "      + std::to_string(get_charge_a()    ) + "\n";
  repr += "CHARGE_B "      + std::to_string(get_charge_b()    ) + "\n";
  repr += "CHARGE_MIN_I "  + std::to_string(get_charge_min_i()) + "\n";
  repr += "X_POS "         + std::to_string(get_x_pos()       ) + "\n";
  repr += "T_AVG "         + std::to_string(get_t_avg()       ) + "\n";
  repr += "CTR_ETX "       + std::to_string(ctr_etx           ) + "\n";
  repr += "TAIL "          + std::to_string(tail              ) + "\n"; 
  return repr;
}

/*******************************************/

bool RPaddlePacket::is_broken() {
  return broken;
}

/*******************************************/

std::ostream& operator<<(std::ostream& os, const RPaddlePacket& pad)
{
   os << pad.to_string();
   return os;
}

void RPaddlePacket::set_time_a(double time)
{
  f32 prec = 0.004;
  time_a = (uint16_t) (time/prec);
}
void RPaddlePacket::set_time_b(double time)
{
  f32 prec = 0.004;
  time_b = (uint16_t) (time/prec);
}
void RPaddlePacket::set_peak_a(double peak)
{
  f32 prec = 0.2;
  peak_a = (uint16_t) (peak/prec);
}
void RPaddlePacket::set_peak_b(double peak)
{
  f32 prec = 0.2;
  peak_b = (uint16_t) (peak/prec);
}
void RPaddlePacket::set_charge_a(double charge)
{
  f32 prec = 0.01; //pC
  charge_a = (uint16_t)(charge/prec - 50);

}
void RPaddlePacket::set_charge_b(double charge)
{
  f32 prec = 0.01; //pC
  charge_b = (uint16_t)(charge/prec - 50);
}
void RPaddlePacket::set_charge_min_i(double charge)
{

}
void RPaddlePacket::set_x_pos(double pos)
{
  f32 prec = 0.005; //cm
  x_pos = (uint16_t)(pos/prec - 163.8);

}
void RPaddlePacket::set_t_avg(double t)
{

}

