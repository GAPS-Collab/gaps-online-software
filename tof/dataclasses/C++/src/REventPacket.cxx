#include "parsers.h"
#include "serialization.h"
#include "packets/REventPacket.h"
#include <spdlog/spdlog.h>

#include "tof_typedefs.h"

void REventPacket::reset()
{
   head = 0xAAAA;

   // eventualy, maybe
   //p_length_fixed = REVENTPACKETSIZEFIXED;
   p_length_fixed      = 15; // currently no primary information in stream
   n_paddles           = 0;
   event_ctr           = 0;
   timestamp_32        = 0;
   timestamp_16        = 0;

   primary_beta        = 0;
   primary_beta_unc    = 0;
   primary_charge      = 0;
   primary_charge_unc  = 0;
   primary_outer_tof_x = 0;
   primary_outer_tof_y = 0;
   primary_outer_tof_z = 0;
   primary_inner_tof_x = 0;
   primary_inner_tof_y = 0;
   primary_inner_tof_z = 0;

   nhit_outer_tof = 0x00      ;
   nhit_inner_tof = 0x00      ;

   trigger_info   = 0x00      ;
   ctr_etx        = 0x00      ;

   paddle_info.clear();

   tail = 0x5555;
}

/*******************************************/

u16 REventPacket::calculate_length() const
{

  // currently we have a fixed length + the size of 
  // 1 paddle packet per hit paddle
  
  // fixed part is 42 byte
  return p_length_fixed + (RPaddlePacket::calculate_length()*paddle_info.size());
}

/*******************************************/

void REventPacket::add_paddle_packet(RPaddlePacket const &pkt)
{
  paddle_info.push_back(pkt);
  n_paddles += 1;
}

/*******************************************/

Vec<u8> REventPacket::serialize() const
{
  // this takes into acount variable sized part
  //unsigned short packet_length = calculate_length();
  //std::cout << "Got packet length : " << packet_length << std::endl;
  Vec<u8> buffer(p_length_fixed);
  //std::cout << "Allocated buffer of size " << buffer.size() << std::endl;  

  u8 pos = 0; // position in bytestream
  encode_ushort(head, buffer, pos); pos+=2;
  encode_ushort(n_paddles, buffer, pos); pos+=2;
  encode_uint32(event_ctr, buffer, pos); pos+=4;
  //encode_uint64(utc_timestamp, buffer, pos); pos+=8;
  u32_to_le_bytes(timestamp_32, buffer, pos);
  encode_ushort(timestamp_16, buffer, pos); pos+=2;

  encode_ushort(primary_beta,       buffer,  pos);pos+=2;  
  encode_ushort(primary_beta_unc,   buffer,  pos);pos+=2;  
  encode_ushort(primary_charge,     buffer,  pos);pos+=2;  
  encode_ushort(primary_charge_unc, buffer,  pos);pos+=2;  
  encode_ushort(primary_outer_tof_x, buffer, pos);pos+=2;  
  encode_ushort(primary_outer_tof_y, buffer, pos);pos+=2;  
  encode_ushort(primary_outer_tof_z, buffer, pos);pos+=2;  
  encode_ushort(primary_inner_tof_x, buffer, pos);pos+=2;  
  encode_ushort(primary_inner_tof_y, buffer, pos);pos+=2;  
  encode_ushort(primary_inner_tof_z, buffer, pos);pos+=2;  

  buffer[pos] = nhit_outer_tof; pos+=1;
  buffer[pos] = nhit_inner_tof; pos+=1;
  buffer[pos] = trigger_info;   pos+=1;
  buffer[pos] = ctr_etx;        pos+=1;

  Vec<u8> paddle_payload;
  for (const auto& pinfo : paddle_info)
    {
        paddle_payload = pinfo.serialize();
        buffer.insert(buffer.end(), paddle_payload.begin(), paddle_payload.end()); 
        //std::cout <<  "[DEBUG] added paddle of size " << paddle_payload.size() 
        //              << " bytes to buffer" << std::endl;
        pos += paddle_payload.size();
    } 
  encode_ushort(tail, buffer, pos); pos+=2;  // done
  return buffer; 
}

/*******************************************/


//! FIXME - deserialize all fields.
//  currently decode only paddle packets 
//  (the reconstructed primary is not available anyway)
u32 REventPacket::deserialize(Vec<u8> &bytestream,
                              u64 start_pos)
{
  reset ();
 
  // start from position in bytestream
  //unsigned short value; 
  //unsigned int end_pos = start_pos;
  // check if we find the header at start_pos
  u16 value = Gaps::u16_from_le_bytes(bytestream, start_pos);
  if (value != head)
    {spdlog::error("No header found!");}
  //u64 pos = 2 + start_pos; // position in bytestream, 2 since we 
                    // just decoded the header
  u64 pos = start_pos;
  //unsigned short expected_packet_size = decode_ushort(bytestream, pos);pos+=2;  
  //p_length = expected_packet_size;
  // in the expected packet size, we can see how many 
  // paddle packets we expect
  //unsigned short expected_paddle_packets = (expected_packet_size - p_length_fixed)/RPaddlePacket::calculate_length();
  //unsigned short expected_paddle_packets = 
  //std::cout << "[INFO] Expecting " << expected_paddle_packets << " paddle info objects" << std::endl; 
 
  event_ctr           = Gaps::u32_from_le_bytes(bytestream, pos);
  timestamp_32        = Gaps::u32_from_le_bytes(bytestream, pos);
  timestamp_16        = Gaps::u16_from_le_bytes(bytestream, pos);
  n_paddles           = bytestream[pos]; pos += 1;
  /*
  utc_timestamp       = decode_uint64(bytestream, pos); pos+=8;
  //std::cout << "[INFO] Found timestamp " << utc_timestamp << std::endl;
  primary_beta        = decode_ushort(bytestream,  pos);pos+=2;  
  primary_beta_unc    = decode_ushort(bytestream,  pos);pos+=2;  
  primary_charge      = decode_ushort(bytestream,  pos);pos+=2;  
  primary_charge_unc  = decode_ushort(bytestream,  pos);pos+=2;  
  primary_outer_tof_x = decode_ushort(bytestream,  pos);pos+=2;  
  primary_outer_tof_y = decode_ushort(bytestream,  pos);pos+=2;  
  primary_outer_tof_z = decode_ushort(bytestream,  pos);pos+=2;  
  primary_inner_tof_x = decode_ushort(bytestream,  pos);pos+=2;  
  primary_inner_tof_y = decode_ushort(bytestream,  pos);pos+=2;  
  primary_inner_tof_z = decode_ushort(bytestream,  pos);pos+=2;  
 
  nhit_outer_tof = bytestream[pos];pos+=1;
  nhit_inner_tof = bytestream[pos];pos+=1;
  trigger_info   = bytestream[pos];pos+=1;
  ctr_etx        = bytestream[pos];pos+=1;
  paddle_info.reserve(expected_paddle_packets);
  RPaddlePacket p;
  for (size_t k=0;k<expected_paddle_packets;k++)
    {  
     p.deserialize(bytestream, pos);
     paddle_info.push_back(p);
     pos += p.calculate_length();
    }
  */
 
  // FIXME checks - packetlength, checksum ?
  // check if the trailer is right after the header, 
  int debug_trailer        = Gaps::u16_from_le_bytes(bytestream, pos);
  if (debug_trailer == tail) {
    if (n_paddles > 0) {
      broken = true;
    }
    return pos;
  }
  pos -= 2; // do not advance, that was just a check

  paddle_info.reserve(n_paddles);
  RPaddlePacket p;
  u8 paddles_found = 0;
  while (paddles_found < n_paddles) {
    p.deserialize(bytestream, pos);
    if (!p.is_broken()) {
      paddle_info.push_back(p);
      pos += p.calculate_length();
      paddles_found += 1;
    } else {
      paddle_info.push_back(p);
      paddles_found += 1;
      //std::cout << "BROKEN " << p << std::endl;
      // we stop at the first broken package
      break;
    }
  }
  u16 payload_tail = Gaps::u32_from_le_bytes(bytestream, pos);
  if (payload_tail != tail) //|| (expected_packet_size != pos))
     {
        spdlog::error("Broken package! Tail flag is not correct!");
        //std::cerr << "[ERROR] broken package! Tail flag "<< payload_tail 
        //    //<< " expected size " << expected_packet_size 
        //    << " received " << pos << " bytes!" << std::endl;
        broken = true;
     }

  if (paddle_info.size() != n_paddles) {
    broken = true;
  }
  return pos; 
}

std::string REventPacket::to_string(bool summarize_paddle_packets) const
{
  std::string output;
  output += "### REVENTPACKET-----------------------\n";
  output += "\tHEAD \t"                + std::to_string(head) +  "\n";
  output += "\tEVENT CTR \t"           + std::to_string(event_ctr)         + "\n";
  output += "\tTIMESTAMP 32 \t"        + std::to_string(timestamp_32)     + "\n";
  output += "\tTIMESTAMP 16 \t"        + std::to_string(timestamp_16)     + "\n";
  output += "\tN PADDLES \t"           + std::to_string(n_paddles) + "\n";
  output += "\tPRIMARY BETA \t"        + std::to_string(primary_beta)     + "\n";
  output += "\tPRIMARY BETA UNC \t"    + std::to_string(primary_beta_unc) + "\n";
  output += "\tPRIMARY CHARGE \t"      + std::to_string(primary_charge)   + "\n";
  output += "\tPRIMARY CHARGE UNC \t"  + std::to_string(primary_charge_unc)  + "\n";
  output += "\tPRIMARY OUTER TOF X \t" + std::to_string(primary_outer_tof_x) + "\n";
  output += "\tPRIMARY OUTER TOF Y \t" + std::to_string(primary_outer_tof_y) + "\n";
  output += "\tPRIMARY OUTER TOR Z \t" + std::to_string(primary_outer_tof_z) + "\n";
  output += "\tPRIMARY INNER TOF X \t" + std::to_string(primary_inner_tof_x) + "\n";
  output += "\tPRIMARY INNER TOF Y \t" + std::to_string(primary_inner_tof_y) + "\n";
  output += "\tPRIMARY INNER TOF Z \t" + std::to_string(primary_inner_tof_z) + "\n";
  output += "\tNHIT OUTER \t"          + std::to_string(nhit_outer_tof) + "\n";
  output += "\tNHIT INNER \t"          + std::to_string(nhit_inner_tof) + "\n";
  output += "\tTRG INFO \t"            + std::to_string(trigger_info) + "\n";
  output += "\tCTR ETX \t"             + std::to_string(ctr_etx) + "\n";
  output += "\tNPADDLE PACKETS \t"     + std::to_string(paddle_info.size()) + "\n";
  if (summarize_paddle_packets) 
   { output += "\t[ " + std::to_string(paddle_info.size()) + " PADDLE PACKETS ... ]\n";} 
  else {
   for (auto p : paddle_info) 
     { output +=  p.to_string() + "\n";}
  }
  output += "\tTAIL \t" + std::to_string(tail) + "\n";
  return output;
}

bool REventPacket::is_broken()
{
  return broken;
}

std::ostream& operator<<(std::ostream& os, const REventPacket& evt)
{
   os << evt.to_string();
   return os;
}


