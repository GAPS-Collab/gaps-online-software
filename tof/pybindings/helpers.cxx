#include "tof_typedefs.h"
#include "serialization.h"
#include "io.hpp"

bytestream wrap_encode_ushort(u16 value, u32 start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort_rev(value, stream, start_pos);
  return stream;
}


/***********************************************/

bytestream wrap_u32_to_le_bytes(u32 value) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  u32_to_le_bytes(value, stream, 0);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64(value, stream, start_pos);
  return stream;
}

/***********************************************/

Vec<TofPacket> wrap_get_tofpacket_from_file(const String filename) {
  return get_tofpackets(filename);
}

Vec<TofPacket> wrap_get_tofpacket_from_stream(const Vec<u8> &stream, u64 pos) {
  return get_tofpackets(stream, pos);
}

/***********************************************/

String rbmoni_to_string(const RBMoniData &moni) {
  String repr = "<RBMoniData: \n";
  repr += "\t board_id           " + std::to_string(moni.board_id)         + "\n"; 
  repr += "\t rate               " + std::to_string(moni.rate)             + "\n"; 
  repr += "\t tmp_drs            " + std::to_string(moni.tmp_drs)          + "\n"; 
  repr += "\t tmp_clk            " + std::to_string(moni.tmp_clk)          + "\n"; 
  repr += "\t tmp_adc            " + std::to_string(moni.tmp_adc)          + "\n"; 
  repr += "\t tmp_zynq           " + std::to_string(moni.tmp_zynq)         + "\n"; 
  repr += "\t tmp_lis3mdltr      " + std::to_string(moni.tmp_lis3mdltr)    + "\n"; 
  repr += "\t tmp_bm280          " + std::to_string(moni.tmp_bm280)        + "\n"; 
  repr += "\t pressure           " + std::to_string(moni.pressure)         + "\n"; 
  repr += "\t humidity           " + std::to_string(moni.humidity)         + "\n"; 
  repr += "\t mag_x              " + std::to_string(moni.mag_x)            + "\n"; 
  repr += "\t mag_y              " + std::to_string(moni.mag_y)            + "\n"; 
  repr += "\t mag_z              " + std::to_string(moni.mag_z)            + "\n"; 
  repr += "\t mag_tot            " + std::to_string(moni.mag_tot)          + "\n"; 
  repr += "\t drs_dvdd_voltage   " + std::to_string(moni.drs_dvdd_voltage) + "\n"; 
  repr += "\t drs_dvdd_current   " + std::to_string(moni.drs_dvdd_current) + "\n"; 
  repr += "\t drs_dvdd_power     " + std::to_string(moni.drs_dvdd_power)   + "\n"; 
  repr += "\t p3v3_voltage       " + std::to_string(moni.p3v3_voltage)     + "\n"; 
  repr += "\t p3v3_current       " + std::to_string(moni.p3v3_current)     + "\n"; 
  repr += "\t p3v3_power         " + std::to_string(moni.p3v3_power)       + "\n"; 
  repr += "\t zynq_voltage       " + std::to_string(moni.zynq_voltage)     + "\n"; 
  repr += "\t zynq_current       " + std::to_string(moni.zynq_current)     + "\n"; 
  repr += "\t zynq_power         " + std::to_string(moni.zynq_power)       + "\n"; 
  repr += "\t p3v5_voltage       " + std::to_string(moni.p3v5_voltage)     + "\n";  
  repr += "\t p3v5_current       " + std::to_string(moni.p3v5_current)     + "\n"; 
  repr += "\t p3v5_power         " + std::to_string(moni.p3v5_power)       + "\n"; 
  repr += "\t adc_dvdd_voltage   " + std::to_string(moni.adc_dvdd_voltage) + "\n"; 
  repr += "\t adc_dvdd_current   " + std::to_string(moni.adc_dvdd_current) + "\n"; 
  repr += "\t adc_dvdd_power     " + std::to_string(moni.adc_dvdd_power)   + "\n"; 
  repr += "\t adc_avdd_voltage   " + std::to_string(moni.adc_avdd_voltage) + "\n"; 
  repr += "\t adc_avdd_current   " + std::to_string(moni.adc_avdd_current) + "\n"; 
  repr += "\t adc_avdd_power     " + std::to_string(moni.adc_avdd_power)   + "\n"; 
  repr += "\t drs_avdd_voltage   " + std::to_string(moni.drs_avdd_voltage) + "\n"; 
  repr += "\t drs_avdd_current   " + std::to_string(moni.drs_avdd_current) + "\n"; 
  repr += "\t drs_avdd_power     " + std::to_string(moni.drs_avdd_power)   + "\n"; 
  repr += "\t n1v5_voltage       " + std::to_string(moni.n1v5_voltage)     + "\n"; 
  repr += "\t n1v5_current       " + std::to_string(moni.n1v5_current)     + "\n"; 
  repr += "\t n1v5_power         " + std::to_string(moni.n1v5_power)       + "\n"; 
  repr += " >";
  return repr;
}

