#include <format>

#include "packets/monitoring.h"
#include "parsers.h"
#include "logging.hpp"

LTBMoniData::LTBMoniData() {
  board_id          = 0;
  trenz_temp        = 0;
  ltb_temp          = 0;
  thresh            = {0,0,0};
}

LTBMoniData LTBMoniData::from_bytestream(const Vec<u8> &stream,
                                         usize &pos) {
  auto moni = LTBMoniData();
  u16 head          = Gaps::parse_u16(stream, pos);
  if (head != LTBMoniData::HEAD) {
    log_error("No header signature (0xAAAA) found for decoding of LTBMoniData!");   
  }
  moni.board_id    = Gaps::parse_u8(stream, pos);
  moni.trenz_temp  = Gaps::parse_f32(stream, pos);
  moni.ltb_temp    = Gaps::parse_f32(stream, pos);
  for (usize k=0;k<3;k++) {
    moni.thresh[k] = Gaps::parse_f32(stream, pos);
  }
  u16 tail         = Gaps::parse_u16(stream, pos);
  if (tail != LTBMoniData::TAIL) {
    log_error("No tail signature (0x5555) found for decoding of LTBMoniData!");   
  }
  return moni;
}
  
std::string LTBMoniData::to_string() const {
  std::string repr = "<LTBMoniData   : ";
  repr += std::format("\n  board_id              : {}"      ,board_id   );
  repr += std::format("\n  trenz temp      [\u00B0C]  : {:.2}" ,trenz_temp );
  repr += std::format("\n  LTB   temp      [\u00B0C]  : {:.2}" ,ltb_temp   );
  repr += "\n  ** Thresholds **";
  repr += "\n  THR1, THR2, THR3 [mV] : " + std::to_string(thresh[0]) 
       +  " " + std::to_string(thresh[1])
       +  " " + std::to_string(thresh[2]);
  repr += ">";
  return repr;
}


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

RBMoniData RBMoniData::from_bytestream(const Vec<u8> &payload,
                                       usize &pos) {
  //usize pos = start_pos; 
  RBMoniData moni   = RBMoniData();
  u16 head          = Gaps::parse_u16(payload, pos);
  if (head != RBMoniData::HEAD) {
    log_error("No header signature (0xAAAA) found for decoding of RBMoniData!");   
  }
  moni.board_id          = Gaps::parse_u8(payload, pos);  
  moni.rate              = Gaps::parse_u16(payload, pos);  
  moni.tmp_drs           = Gaps::parse_f32(payload, pos);  
  moni.tmp_clk           = Gaps::parse_f32(payload, pos);  
  moni.tmp_adc           = Gaps::parse_f32(payload, pos);  
  moni.tmp_zynq          = Gaps::parse_f32(payload, pos);  
  moni.tmp_lis3mdltr     = Gaps::parse_f32(payload, pos);  
  moni.tmp_bm280         = Gaps::parse_f32(payload, pos);  
  moni.pressure          = Gaps::parse_f32(payload, pos);  
  moni.humidity          = Gaps::parse_f32(payload, pos);  
  moni.mag_x             = Gaps::parse_f32(payload, pos);  
  moni.mag_y             = Gaps::parse_f32(payload, pos);  
  moni.mag_z             = Gaps::parse_f32(payload, pos);  
  moni.mag_tot           = Gaps::parse_f32(payload, pos);  
  moni.drs_dvdd_voltage  = Gaps::parse_f32(payload, pos);  
  moni.drs_dvdd_current  = Gaps::parse_f32(payload, pos);  
  moni.drs_dvdd_power    = Gaps::parse_f32(payload, pos);  
  moni.p3v3_voltage      = Gaps::parse_f32(payload, pos);  
  moni.p3v3_current      = Gaps::parse_f32(payload, pos);  
  moni.p3v3_power        = Gaps::parse_f32(payload, pos);  
  moni.zynq_voltage      = Gaps::parse_f32(payload, pos);  
  moni.zynq_current      = Gaps::parse_f32(payload, pos);  
  moni.zynq_power        = Gaps::parse_f32(payload, pos);  
  moni.p3v5_voltage      = Gaps::parse_f32(payload, pos);  
  moni.p3v5_current      = Gaps::parse_f32(payload, pos);  
  moni.p3v5_power        = Gaps::parse_f32(payload, pos);  
  moni.adc_dvdd_voltage  = Gaps::parse_f32(payload, pos);  
  moni.adc_dvdd_current  = Gaps::parse_f32(payload, pos);  
  moni.adc_dvdd_power    = Gaps::parse_f32(payload, pos);  
  moni.adc_avdd_voltage  = Gaps::parse_f32(payload, pos);  
  moni.adc_avdd_current  = Gaps::parse_f32(payload, pos);  
  moni.adc_avdd_power    = Gaps::parse_f32(payload, pos);  
  moni.drs_avdd_voltage  = Gaps::parse_f32(payload, pos);  
  moni.drs_avdd_current  = Gaps::parse_f32(payload, pos);  
  moni.drs_avdd_power    = Gaps::parse_f32(payload, pos);  
  moni.n1v5_voltage      = Gaps::parse_f32(payload, pos);  
  moni.n1v5_current      = Gaps::parse_f32(payload, pos);  
  moni.n1v5_power        = Gaps::parse_f32(payload, pos);  
  u16 tail               = Gaps::parse_u16(payload, pos);
  if (tail != RBMoniData::TAIL) {
    log_error("No tail signature (0x5555) found for decoding of RBMoniData!");   
  }
  return moni;
}
  
std::string RBMoniData::to_string() const {
  std::string repr = "<RBMoniData : ";
  repr += "\n board_id           : " + std::to_string(board_id         );
  repr += "\n rate               : " + std::to_string(rate             );
  repr += "\n tmp_drs            : " + std::to_string(tmp_drs          );
  repr += "\n tmp_clk            : " + std::to_string(tmp_clk          );
  repr += "\n tmp_adc            : " + std::to_string(tmp_adc          );
  repr += "\n tmp_zynq           : " + std::to_string(tmp_zynq         );
  repr += "\n tmp_lis3mdltr      : " + std::to_string(tmp_lis3mdltr    );
  repr += "\n tmp_bm280          : " + std::to_string(tmp_bm280        );
  repr += "\n pressure           : " + std::to_string(pressure         );
  repr += "\n humidity           : " + std::to_string(humidity         );
  repr += "\n mag_x              : " + std::to_string(mag_x            );
  repr += "\n mag_y              : " + std::to_string(mag_y            );
  repr += "\n mag_z              : " + std::to_string(mag_z            );
  repr += "\n mag_tot            : " + std::to_string(mag_tot          );
  repr += "\n drs_dvdd_voltage   : " + std::to_string(drs_dvdd_voltage );
  repr += "\n drs_dvdd_current   : " + std::to_string(drs_dvdd_current );
  repr += "\n drs_dvdd_power     : " + std::to_string(drs_dvdd_power   );
  repr += "\n p3v3_voltage       : " + std::to_string(p3v3_voltage     );
  repr += "\n p3v3_current       : " + std::to_string(p3v3_current     );
  repr += "\n p3v3_power         : " + std::to_string(p3v3_power       );
  repr += "\n zynq_voltage       : " + std::to_string(zynq_voltage     );
  repr += "\n zynq_current       : " + std::to_string(zynq_current     );
  repr += "\n zynq_power         : " + std::to_string(zynq_power       );
  repr += "\n p3v5_voltage       : " + std::to_string(p3v5_voltage     );
  repr += "\n p3v5_current       : " + std::to_string(p3v5_current     );
  repr += "\n p3v5_power         : " + std::to_string(p3v5_power       );
  repr += "\n adc_dvdd_voltage   : " + std::to_string(adc_dvdd_voltage );
  repr += "\n adc_dvdd_current   : " + std::to_string(adc_dvdd_current );
  repr += "\n adc_dvdd_power     : " + std::to_string(adc_dvdd_power   );
  repr += "\n adc_avdd_voltage   : " + std::to_string(adc_avdd_voltage );
  repr += "\n adc_avdd_current   : " + std::to_string(adc_avdd_current );
  repr += "\n adc_avdd_power     : " + std::to_string(adc_avdd_power   );
  repr += "\n drs_avdd_voltage   : " + std::to_string(drs_avdd_voltage );
  repr += "\n drs_avdd_current   : " + std::to_string(drs_avdd_current );
  repr += "\n drs_avdd_power     : " + std::to_string(drs_avdd_power   );
  repr += "\n n1v5_voltage       : " + std::to_string(n1v5_voltage     );
  repr += "\n n1v5_current       : " + std::to_string(n1v5_current     );
  repr += "\n n1v5_power         : " + std::to_string(n1v5_power       );
  repr += ">";
  return repr;
}

PBMoniData::PBMoniData() {
  board_id         = 0;
  p3v6_preamp_vcp  = {0,0,0};
  n1v6_preamp_vcp  = {0,0,0};
  p3v4f_ltb_vcp    = {0,0,0};
  p3v4d_ltb_vcp    = {0,0,0};
  p3v6_ltb_vcp     = {0,0,0};
  n1v6_ltb_vcp     = {0,0,0};
  pds_temp         = 0;
  pas_temp         = 0;
  nas_temp         = 0;
  shv_temp         = 0;
}

PBMoniData PBMoniData::from_bytestream(const Vec<u8> &stream,
                                       usize &pos) {
  auto moni = PBMoniData();
  u16 head             = Gaps::parse_u16(stream, pos);
  if (head != PBMoniData::HEAD) {
    log_error("No header signature (0xAAAA) found for decoding of PBMoniData!");   
  }
  moni.board_id         = Gaps::parse_u8(stream, pos);
  for (auto k : {0,1,2}) {
    moni.p3v6_preamp_vcp[k]  = Gaps::parse_f32(stream, pos);
  }
  for (auto k : {0,1,2}) {
    moni.n1v6_preamp_vcp[k]  = Gaps::parse_f32(stream, pos);
  }
  for (auto k : {0,1,2}) {
    moni.p3v4f_ltb_vcp[k]    = Gaps::parse_f32(stream, pos);
  }
  for (auto k : {0,1,2}) {
    moni.p3v4d_ltb_vcp[k]    = Gaps::parse_f32(stream, pos);
  }
  for (auto k : {0,1,2}) {
    moni.p3v6_ltb_vcp[k]     = Gaps::parse_f32(stream, pos);
  }
  for (auto k : {0,1,2}) {
    moni.n1v6_ltb_vcp[k]     = Gaps::parse_f32(stream, pos);
  }
  moni.pds_temp         = Gaps::parse_f32(stream, pos);
  moni.pas_temp         = Gaps::parse_f32(stream, pos);
  moni.nas_temp         = Gaps::parse_f32(stream, pos);
  moni.shv_temp         = Gaps::parse_f32(stream, pos);
  u16 tail              = Gaps::parse_u16(stream, pos);
  if (tail != PBMoniData::TAIL) {
    log_error("No tail signature (0x5555) found for decoding of PBMoniData!");   
  }
  return moni;
}

std::string PBMoniData::to_string() const {
  std::string repr = "<PBMoniData :";
  repr += "\n board_id        : " + std::to_string(board_id         );
  repr += "\n p3v6_preamp_vcp : " + std::to_string( p3v6_preamp_vcp [0] )+ std::to_string( p3v6_preamp_vcp [1] )+ std::to_string( p3v6_preamp_vcp [2] );
  repr += "\n n1v6_preamp_vcp : " + std::to_string( n1v6_preamp_vcp [0] )+ std::to_string( n1v6_preamp_vcp [1] )+ std::to_string( n1v6_preamp_vcp [2] );
  repr += "\n p3v4f_ltb_vcp   : " + std::to_string( p3v4f_ltb_vcp   [0] )+ std::to_string( p3v4f_ltb_vcp   [1] )+ std::to_string( p3v4f_ltb_vcp   [2] );
  repr += "\n p3v4d_ltb_vcp   : " + std::to_string( p3v4d_ltb_vcp   [0] )+ std::to_string( p3v4d_ltb_vcp   [1] )+ std::to_string( p3v4d_ltb_vcp   [2] );
  repr += "\n p3v6_ltb_vcp    : " + std::to_string( p3v6_ltb_vcp    [0] )+ std::to_string( p3v6_ltb_vcp    [1] )+ std::to_string( p3v6_ltb_vcp    [2] );
  repr += "\n n1v6_ltb_vcp    : " + std::to_string( n1v6_ltb_vcp    [0] )+ std::to_string( n1v6_ltb_vcp    [1] )+ std::to_string( n1v6_ltb_vcp    [2] );
  repr += "\n pds_temp        : " + std::to_string( pds_temp         );
  repr += "\n pas_temp        : " + std::to_string( pas_temp         );
  repr += "\n nas_temp        : " + std::to_string( nas_temp         );
  repr += "\n shv_temp        : " + std::to_string( shv_temp         );
  return repr;
}

PAMoniData::PAMoniData() {
  board_id = 0;
  temps.fill(0);
  biases.fill(0);
}

PAMoniData PAMoniData::from_bytestream(const Vec<u8> &stream,
                                       usize &pos) {
  auto moni            = PAMoniData();
  u16 head             = Gaps::parse_u16(stream, pos);
  if (head != PAMoniData::HEAD) {
    log_error("No header signature (0xAAAA) found for decoding of PAMoniData!");   
  }
  moni.board_id        = Gaps::parse_u8(stream, pos);
  for (usize k=0;k<16;k++) {
    moni.temps[k]      = Gaps::parse_f32(stream, pos);
  }
  for (usize k=0;k<16;k++) {
    moni.biases[k]     = Gaps::parse_f32(stream, pos);  
  }
  u16 tail             = Gaps::parse_u16(stream, pos);
  if (tail != PAMoniData::TAIL) {
    log_error("No tail signature (0x5555) found for decoding of PAMoniData!");   
  }
  return moni;
}

std::string PAMoniData::to_string() const {
  //repr += std::format("\n  LTB   temp      [\u00B0C]  : {:.2}" ,ltb_temp   );
  std::string repr = "<PAMoniData :";
  repr += std::format("\n  Board ID    : {}", board_id);
  repr += "\n  **temps (16x) [\u00B0C] ";
  for (const auto &k : temps) {
    repr += std::format("{} | ", k); 
  }
  repr += "\n  **biases (16x) [V] ";
  for (const auto &k : biases) {
    repr += std::format("{} | ", k); 
  }
  repr += ">";
  return repr;
}

MtbMoniData::MtbMoniData() {
  fpga_temp    = 0;
  fpga_vccint  = 0;
  fpga_vccaux  = 0;
  fpga_vccbram = 0;
  rate         = 0;
  lost_rate    = 0;
}

MtbMoniData MtbMoniData::from_bytestream(const Vec<u8> &payload,
                                         usize& pos) {
  auto moni = MtbMoniData();
  u16 head          = Gaps::parse_u16(payload, pos);
  if (head != MtbMoniData::HEAD) {
    log_error("No header signature (0xAAAA) found for decoding of MtbMoniData!");   
  }
  moni.tiu_busy_len  = Gaps::parse_u32(payload, pos);
  moni.tiu_status    = Gaps::parse_u8( payload, pos);
  moni.prescale_pc   = Gaps::parse_u8( payload, pos);
  moni.daq_queue_len = Gaps::parse_u16(payload, pos);
  moni.fpga_temp     = Gaps::parse_u16(payload, pos);
  moni.fpga_vccint   = Gaps::parse_u16(payload, pos);
  moni.fpga_vccaux   = Gaps::parse_u16(payload, pos);
  moni.fpga_vccbram  = Gaps::parse_u16(payload, pos);
  moni.rate          = Gaps::parse_u16(payload, pos);
  moni.lost_rate     = Gaps::parse_u16(payload, pos);
  u16 tail           = Gaps::parse_u16(payload, pos);
  if (tail != MtbMoniData::TAIL) {
    log_error("No tail signature (0x5555) found for decoding of MtbMoniData!");   
  }
  return moni;
}

bool MtbMoniData::get_tiu_emulation_mode() const {
  return (tiu_status & 0x1) > 0;
}

bool MtbMoniData::get_tiu_use_aux_link() const {
  return (tiu_status & 0x2) > 0;
}

bool MtbMoniData::get_tiu_bad() const { 
  return (tiu_status & 0x4) > 0;
}

bool MtbMoniData::get_tiu_busy_stuck() const {
  return (tiu_status & 0x8) > 0;
}

bool MtbMoniData::get_tiu_ignore_busy() const {
  return (tiu_status & 0x10) > 0;
}
  
f32 MtbMoniData::get_fpga_temp() const {
  return (f32)fpga_temp * 503.975 / 4096.0 - 273.15;
}

  /// Convert ADC temp from adc values to Celsius
  //pub fn get_fpga_temp(&self) -> f32 {
  //  self.temp as f32 * 503.975 / 4096.0 - 273.15
  //}

std::string MtbMoniData::to_string() const {
  //  write!(f, "<MtbMoniData:
  //MTB  EVT RATE  [Hz] {}
  //LOST EVT RATE  [Hz] {}
  //TIU BUSY CNT  [CLK] {}
  //DAQ QUEUE LEN       {}
  //PRESCALE        [%] {}
  //--- TIU STATUS ---
  //  EMU MODE          {}
  //  USE AUX LINK      {}
  //  TIU BAD           {}
  //  BUSY STUCK        {}
  //  IGNORE BUSY       {}
  //--- --- --- --- --
  //FPGA TEMP      [\u{00B0}C] {:.2}
  //VCCINT          [V] {:.3}
  //VCCAUX          [V] {:.3}
  //VCCBRAM         [V] {:.3}>",
  std::string repr = "<MtbMoniData :";
  repr += "\n rate         :" + std::to_string(rate          );
  repr += "\n lost_rate    :" + std::to_string(lost_rate     );
  repr += std::format("\n  fpga temp   [\u00B0C] : {:.2}", get_fpga_temp()); 
  repr += ">";
  return repr; 
}


CPUMoniData::CPUMoniData() {
  uptime     = 0; 
  disk_usage = 0; 
  cpu_freq   = {0,0,0,0};
  cpu_temp   = 9999;
  cpu0_temp  = 9999;
  cpu1_temp  = 9999;
  mb_temp    = 9999;
}

CPUMoniData CPUMoniData::from_bytestream(const Vec<u8> &stream,
                                               usize &pos) {
  auto moni = CPUMoniData();
  u16 head  = Gaps::parse_u16(stream, pos);
  if (head != CPUMoniData::HEAD) {
    log_error("No header signature (0xAAAA) found for decoding of CPUMoniData!");   
  }
  moni.uptime        = Gaps::parse_u32(stream, pos); 
  moni.disk_usage    = Gaps::parse_u8(stream, pos); 
  for (usize k : {0,1,2,3}) {
    moni.cpu_freq[k] = Gaps::parse_u32(stream, pos);
  }
  moni.cpu_temp   = Gaps::parse_f32(stream, pos);
  moni.cpu0_temp  = Gaps::parse_f32(stream, pos);
  moni.cpu1_temp  = Gaps::parse_f32(stream, pos);
  moni.mb_temp    = Gaps::parse_f32(stream, pos);
  u16 tail  = Gaps::parse_u16(stream, pos);
  if (tail != CPUMoniData::TAIL) {
    log_error("No tail signature (0x5555) found for decoding of CPUMoniData!");   
  }
  return moni;
}

std::string CPUMoniData::to_string() const {
  std::string repr = "<CPUMoniData:";
  repr += std::format("\n  core0   T    [\u00B0C] : {:.2}", cpu0_temp); 
  repr += std::format("\n  core1   T    [\u00B0C] : {:.2}", cpu1_temp); 
  repr += std::format("\n  CPU     T    [\u00B0C] : {:.2}", cpu_temp); 
  repr += std::format("\n  MB      T    [\u00B0C] : {:.2}", mb_temp); 
  repr += std::format("\n  CPU (4) freq [Hz] : {} | {} | {} | {}", cpu_freq[0], cpu_freq[1], cpu_freq[2], cpu_freq[3]); 
  repr += std::format("\n  Disc usage    [%] : {}", disk_usage); 
  repr += std::format("\n  Uptime        [s] : {}>", uptime   );
  return repr;
}

std::ostream& operator<<(std::ostream& os, const CPUMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const LTBMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const RBMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const PBMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const PAMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const MtbMoniData& moni){
  os << moni.to_string();
  return os;
}

