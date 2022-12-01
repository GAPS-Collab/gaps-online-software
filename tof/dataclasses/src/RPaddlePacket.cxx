#include "packets/RPaddlePacket.h"
#include "serialization.h"

void RPaddlePacket::reset()
{
  //head = 0xAAAA;
  // define a different header
  head = 0xF0F0;
  p_length = RPADDLEPACKETSIZE;
  //event_ctr = 0;
  //for (size_t k=0;k<8;k++) utc_timestamp[k] = 0x00;

  paddle_id = 0x00;
  time_a = 0;
  time_b = 0;
  peak_a = 0;
  peak_b = 0;
  charge_a = 0;
  charge_b = 0;
  charge_min_i = 0;
  x_pos = 0;
  t_average = 0;
  /// also different tail here, 
  //  so we can find the packet in 
  //  the REventstream
  ctr_etx = 0x00;
  tail = 0xF0F;

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
unsigned short RPaddlePacket::get_paddle_id() const
{
   return (unsigned short)paddle_id;
}

float RPaddlePacket::get_time_a() const
{
   float prec = 0.004;
   return prec*time_a;
}

float RPaddlePacket::get_time_b() const
{
  float prec = 0.004;//ns
  return prec*time_b;
}

float RPaddlePacket::get_peak_a() const
{
  float prec = 0.2;
  return prec*peak_a;
}

float RPaddlePacket::get_peak_b() const
{
  float prec = 0.2;
  return prec*peak_b;
}

float RPaddlePacket::get_charge_a() const
{
  float prec = 0.01; //pC
  return prec*charge_a - 50;
}

float RPaddlePacket::get_charge_b() const
{
  float prec = 0.01;
  return prec*charge_b - 50;
}

float RPaddlePacket::get_charge_min_i() const
{
  float prec = 0.002;// minI
  return prec*charge_min_i - 10;
}

float RPaddlePacket::get_x_pos() const
{
  // FIXME - check if it is really in the middle
  float prec = 0.005; //cm
  return prec*x_pos - 163.8;
}

float RPaddlePacket::get_t_avg() const
{
  float prec = 0.004;//ps
  return prec*t_average;
}


/*******************************************/

std::vector<unsigned char> RPaddlePacket::serialize() const
{
  unsigned short packet_length = calculate_length();
  std::vector<unsigned char> buffer(packet_length);
  unsigned short pos = 0; // position in bytestream
  encode_ushort(head, buffer, pos); pos+=2;
  encode_ushort(p_length, buffer, pos); pos+=2;
  //encode_uint32(event_ctr, buffer, pos); pos+=4;
  buffer[pos] = paddle_id; pos+=1;

  //encode_ushort(paddle_id, buffer, pos); pos+=2;
  encode_ushort(time_a, buffer, pos); pos +=2;
  encode_ushort(time_b, buffer, pos); pos +=2;
  encode_ushort(peak_a, buffer, pos); pos +=2;
  encode_ushort(peak_b, buffer, pos); pos +=2;
  encode_ushort(charge_a, buffer, pos); pos+=2;
  encode_ushort(charge_b, buffer, pos); pos+=2;
  encode_ushort(charge_min_i, buffer, pos); pos+=2;
  encode_ushort(x_pos, buffer, pos); pos+=2;
  encode_ushort(t_average, buffer, pos); pos+=2;

  buffer[pos] = ctr_etx;        pos+=1;

  encode_ushort(tail, buffer, pos); pos+=2;  // done
  return buffer; 
}

/*******************************************/

unsigned int RPaddlePacket::deserialize(std::vector<unsigned char>& bytestream,
                                        unsigned int start_pos)
{
 // start from position in bytestream
 unsigned short value; 
 unsigned int end_pos = start_pos;

 // find start marker in bytestream
 for (size_t k=start_pos;k<bytestream.size();k++)
 {
   value = decode_ushort(bytestream, start_pos=k);
   if (head == value)
    {
     // end pos should point to the start
     // of the next new value
     end_pos = k+2;
     break;
    }
 }

 unsigned int pos = end_pos; // position in bytestream
 unsigned short expected_packet_size = decode_ushort(bytestream, pos);pos+=2; 

 //event_ctr = decode_uint32(bytestream, pos); pos+=4;

 paddle_id = bytestream[pos]; pos+=1;
 time_a = decode_ushort(bytestream, pos); pos+=2;
 time_b = decode_ushort(bytestream, pos); pos+=2;
 peak_a = decode_ushort(bytestream, pos); pos+=2;
 peak_b = decode_ushort(bytestream, pos); pos+=2;
 charge_a = decode_ushort(bytestream, pos); pos+=2;
 charge_b = decode_ushort(bytestream, pos); pos+=2;
 charge_min_i= decode_ushort(bytestream, pos); pos+=2;
 x_pos = decode_ushort(bytestream, pos);pos+=2;
 t_average = decode_ushort(bytestream, pos);pos+=2;

 ctr_etx = bytestream[pos]; pos+=1;
 // FIXME checks - packetlength, checksum ?

 return pos; 
}

/*******************************************/

std::string RPaddlePacket::to_string() const
{
  std::string repr = "";
  repr += "RPADDLEPACKET-----------------------\n";
  repr += "HEAD "          + std::to_string(head              ) + "\n";
  repr += "PACKET_LENGTH " + std::to_string(p_length          ) + "\n";
  //repr += "EVENT CTR "     + std::to_string(event_ctr         ) + "\n";
  //repr += "UTC TS "        + std::to_string(utc_timestamp     ) + "\n";
  repr += "PADDLE ID "     + std::to_string(get_paddle_id()   ) + "\n";
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

std::ostream& operator<<(std::ostream& os, const RPaddlePacket& pad)
{
   os << pad.to_string();
   return os;
}

void RPaddlePacket::set_time_a(double time)
{
  float prec = 0.004;
  time_a = (uint16_t) (time/prec);
}
void RPaddlePacket::set_time_b(double time)
{
  float prec = 0.004;
  time_b = (uint16_t) (time/prec);
}
void RPaddlePacket::set_peak_a(double peak)
{
  float prec = 0.2;
  peak_a = (uint16_t) (peak/prec);
}
void RPaddlePacket::set_peak_b(double peak)
{
  float prec = 0.2;
  peak_b = (uint16_t) (peak/prec);
}
void RPaddlePacket::set_charge_a(double charge)
{
  float prec = 0.01; //pC
  charge_a = (uint16_t)(charge/prec - 50);

}
void RPaddlePacket::set_charge_b(double charge)
{
  float prec = 0.01; //pC
  charge_b = (uint16_t)(charge/prec - 50);
}
void RPaddlePacket::set_charge_min_i(double charge)
{

}
void RPaddlePacket::set_x_pos(double pos)
{
  float prec = 0.005; //cm
  x_pos = (uint16_t)(pos/prec - 163.8);

}
void RPaddlePacket::set_t_avg(double t)
{

}

