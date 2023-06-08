#include "packets/monitoring.h"
#include "parsers.h"

#include "spdlog/spdlog.h"

RBMoniData::RBMoniData() {
  
  board_id          = 0;  
  rate              = 0;  
  tmp_drs           = 0;  
  tmp_clk           = 0;  
  tmp_adc           = 0;  
  tmp_zynq          = 0;  
  tmp_lis3mdltr     = 0;  
  tmp_bm280         = 0;  
  pressure          = 0;  
  humidity          = 0;  
  mag_x             = 0;  
  mag_y             = 0;  
  mag_z             = 0;  
  mag_tot           = 0;  
  drs_dvdd_voltage  = 0;  
  drs_dvdd_current  = 0;  
  drs_dvdd_power    = 0;  
  p3v3_voltage      = 0;  
  p3v3_current      = 0;  
  p3v3_power        = 0;  
  zynq_voltage      = 0;  
  zynq_current      = 0;  
  zynq_power        = 0;  
  p3v5_voltage      = 0;  
  p3v5_current      = 0;  
  p3v5_power        = 0;  
  adc_dvdd_voltage  = 0;  
  adc_dvdd_current  = 0;  
  adc_dvdd_power    = 0;  
  adc_avdd_voltage  = 0;  
  adc_avdd_current  = 0;  
  adc_avdd_power    = 0;  
  drs_avdd_voltage  = 0;  
  drs_avdd_current  = 0;  
  drs_avdd_power    = 0;  
  n1v5_voltage      = 0;  
  n1v5_current      = 0;  
  n1v5_power        = 0;  
}

usize RBMoniData::from_bytestream(Vec<u8> &payload,
                                  usize start_pos) {
  usize pos = start_pos; 
  u16 head          = Gaps::parse_u16(payload, pos);
  if (head != RBMoniData::HEAD) {
    spdlog::error("No header signature (0xAAAA) found for decoding of RBMoniData!");   
  }
  board_id          = Gaps::parse_u8(payload, pos);  
  rate              = Gaps::parse_u16(payload, pos);  
  tmp_drs           = Gaps::parse_f32(payload, pos);  
  tmp_clk           = Gaps::parse_f32(payload, pos);  
  tmp_adc           = Gaps::parse_f32(payload, pos);  
  tmp_zynq          = Gaps::parse_f32(payload, pos);  
  tmp_lis3mdltr     = Gaps::parse_f32(payload, pos);  
  tmp_bm280         = Gaps::parse_f32(payload, pos);  
  pressure          = Gaps::parse_f32(payload, pos);  
  humidity          = Gaps::parse_f32(payload, pos);  
  mag_x             = Gaps::parse_f32(payload, pos);  
  mag_y             = Gaps::parse_f32(payload, pos);  
  mag_z             = Gaps::parse_f32(payload, pos);  
  mag_tot           = Gaps::parse_f32(payload, pos);  
  drs_dvdd_voltage  = Gaps::parse_f32(payload, pos);  
  drs_dvdd_current  = Gaps::parse_f32(payload, pos);  
  drs_dvdd_power    = Gaps::parse_f32(payload, pos);  
  p3v3_voltage      = Gaps::parse_f32(payload, pos);  
  p3v3_current      = Gaps::parse_f32(payload, pos);  
  p3v3_power        = Gaps::parse_f32(payload, pos);  
  zynq_voltage      = Gaps::parse_f32(payload, pos);  
  zynq_current      = Gaps::parse_f32(payload, pos);  
  zynq_power        = Gaps::parse_f32(payload, pos);  
  p3v5_voltage      = Gaps::parse_f32(payload, pos);  
  p3v5_current      = Gaps::parse_f32(payload, pos);  
  p3v5_power        = Gaps::parse_f32(payload, pos);  
  adc_dvdd_voltage  = Gaps::parse_f32(payload, pos);  
  adc_dvdd_current  = Gaps::parse_f32(payload, pos);  
  adc_dvdd_power    = Gaps::parse_f32(payload, pos);  
  adc_avdd_voltage  = Gaps::parse_f32(payload, pos);  
  adc_avdd_current  = Gaps::parse_f32(payload, pos);  
  adc_avdd_power    = Gaps::parse_f32(payload, pos);  
  drs_avdd_voltage  = Gaps::parse_f32(payload, pos);  
  drs_avdd_current  = Gaps::parse_f32(payload, pos);  
  drs_avdd_power    = Gaps::parse_f32(payload, pos);  
  n1v5_voltage      = Gaps::parse_f32(payload, pos);  
  n1v5_current      = Gaps::parse_f32(payload, pos);  
  n1v5_power        = Gaps::parse_f32(payload, pos);  
  u16 tail          = Gaps::parse_u16(payload, pos);
  if (tail != RBMoniData::TAIL) {
    spdlog::error("No tail signature (0x5555) found for decoding of RBMoniData!");   
  }
  return pos;
}

MtbMoniData::MtbMoniData() {
  fpga_temp    = 0 ;
  fpga_vccint  = 0 ;
  fpga_vccaux  = 0 ;
  fpga_vccbram = 0 ;
  rate         = 0 ;
  lost_rate    = 0 ;
}

usize MtbMoniData::from_bytestream(Vec<u8> &payload,
                                   usize start_pos) {
  usize pos = start_pos;
  u16 head          = Gaps::parse_u16(payload, pos);
  if (head != MtbMoniData::HEAD) {
    spdlog::error("No header signature (0xAAAA) found for decoding of MtbMoniData!");   
  }
  fpga_temp    = Gaps::parse_f32(payload, pos);
  fpga_vccint  = Gaps::parse_f32(payload, pos);
  fpga_vccaux  = Gaps::parse_f32(payload, pos);
  fpga_vccbram = Gaps::parse_f32(payload, pos);
  rate         = Gaps::parse_u16(payload, pos);
  lost_rate    = Gaps::parse_u16(payload, pos);
  u16 tail     = Gaps::parse_u16(payload, pos);
  if (tail != MtbMoniData::TAIL) {
    spdlog::error("No tail signature (0x5555) found for decoding of MtbMoniData!");   
  }
  return pos;
}

TofCmpMoniData::TofCmpMoniData() {
  core1_tmp = 0; 
  core2_tmp = 0; 
  pch_tmp   = 0; 
}

usize TofCmpMoniData::from_bytestream(Vec<u8> &payload,
                                      usize start_pos) {
  usize pos = start_pos;
  u16 head  = Gaps::parse_u16(payload, pos);
  if (head != TofCmpMoniData::HEAD) {
    spdlog::error("No header signature (0xAAAA) found for decoding of TofCmpMoniData!");   
  }
  core1_tmp = Gaps::parse_u8(payload, pos); 
  core2_tmp = Gaps::parse_u8(payload, pos); 
  pch_tmp   = Gaps::parse_u8(payload, pos); 
  u16 tail  = Gaps::parse_u16(payload, pos);
  if (tail != TofCmpMoniData::TAIL) {
    spdlog::error("No tail signature (0x5555) found for decoding of TofCmpMoniData!");   
  }
  return pos;
}

