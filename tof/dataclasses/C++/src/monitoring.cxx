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

RBMoniData RBMoniData::from_bytestream(const Vec<u8> &payload,
                                       usize &pos) {
  //usize pos = start_pos; 
  RBMoniData moni = RBMoniData();
  u16 head          = Gaps::parse_u16(payload, pos);
  if (head != RBMoniData::HEAD) {
    spdlog::error("No header signature (0xAAAA) found for decoding of RBMoniData!");   
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
    spdlog::error("No tail signature (0x5555) found for decoding of RBMoniData!");   
  }
  return moni;
}
  
std::string RBMoniData::to_string() const {
  std::string repr = "<RBMoniData : >";
  repr += "\n\t board_id           : " + std::to_string(board_id         );
  repr += "\n\t rate               : " + std::to_string(rate             );
  repr += "\n\t tmp_drs            : " + std::to_string(tmp_drs          );
  repr += "\n\t tmp_clk            : " + std::to_string(tmp_clk          );
  repr += "\n\t tmp_adc            : " + std::to_string(tmp_adc          );
  repr += "\n\t tmp_zynq           : " + std::to_string(tmp_zynq         );
  repr += "\n\t tmp_lis3mdltr      : " + std::to_string(tmp_lis3mdltr    );
  repr += "\n\t tmp_bm280          : " + std::to_string(tmp_bm280        );
  repr += "\n\t pressure           : " + std::to_string(pressure         );
  repr += "\n\t humidity           : " + std::to_string(humidity         );
  repr += "\n\t mag_x              : " + std::to_string(mag_x            );
  repr += "\n\t mag_y              : " + std::to_string(mag_y            );
  repr += "\n\t mag_z              : " + std::to_string(mag_z            );
  repr += "\n\t mag_tot            : " + std::to_string(mag_tot          );
  repr += "\n\t drs_dvdd_voltage   : " + std::to_string(drs_dvdd_voltage );
  repr += "\n\t drs_dvdd_current   : " + std::to_string(drs_dvdd_current );
  repr += "\n\t drs_dvdd_power     : " + std::to_string(drs_dvdd_power   );
  repr += "\n\t p3v3_voltage       : " + std::to_string(p3v3_voltage     );
  repr += "\n\t p3v3_current       : " + std::to_string(p3v3_current     );
  repr += "\n\t p3v3_power         : " + std::to_string(p3v3_power       );
  repr += "\n\t zynq_voltage       : " + std::to_string(zynq_voltage     );
  repr += "\n\t zynq_current       : " + std::to_string(zynq_current     );
  repr += "\n\t zynq_power         : " + std::to_string(zynq_power       );
  repr += "\n\t p3v5_voltage       : " + std::to_string(p3v5_voltage     );
  repr += "\n\t p3v5_current       : " + std::to_string(p3v5_current     );
  repr += "\n\t p3v5_power         : " + std::to_string(p3v5_power       );
  repr += "\n\t adc_dvdd_voltage   : " + std::to_string(adc_dvdd_voltage );
  repr += "\n\t adc_dvdd_current   : " + std::to_string(adc_dvdd_current );
  repr += "\n\t adc_dvdd_power     : " + std::to_string(adc_dvdd_power   );
  repr += "\n\t adc_avdd_voltage   : " + std::to_string(adc_avdd_voltage );
  repr += "\n\t adc_avdd_current   : " + std::to_string(adc_avdd_current );
  repr += "\n\t adc_avdd_power     : " + std::to_string(adc_avdd_power   );
  repr += "\n\t drs_avdd_voltage   : " + std::to_string(drs_avdd_voltage );
  repr += "\n\t drs_avdd_current   : " + std::to_string(drs_avdd_current );
  repr += "\n\t drs_avdd_power     : " + std::to_string(drs_avdd_power   );
  repr += "\n\t n1v5_voltage       : " + std::to_string(n1v5_voltage     );
  repr += "\n\t n1v5_current       : " + std::to_string(n1v5_current     );
  repr += "\n\t n1v5_power         : " + std::to_string(n1v5_power       );
  return repr;
}

MtbMoniData::MtbMoniData() {
  fpga_temp    = 0 ;
  fpga_vccint  = 0 ;
  fpga_vccaux  = 0 ;
  fpga_vccbram = 0 ;
  rate         = 0 ;
  lost_rate    = 0 ;
}

MtbMoniData MtbMoniData::from_bytestream(const Vec<u8> &payload,
                                         usize& pos) {
  auto moni = MtbMoniData();
  u16 head          = Gaps::parse_u16(payload, pos);
  if (head != MtbMoniData::HEAD) {
    spdlog::error("No header signature (0xAAAA) found for decoding of MtbMoniData!");   
  }
  moni.fpga_temp    = Gaps::parse_f32(payload, pos);
  moni.fpga_vccint  = Gaps::parse_f32(payload, pos);
  moni.fpga_vccaux  = Gaps::parse_f32(payload, pos);
  moni.fpga_vccbram = Gaps::parse_f32(payload, pos);
  moni.rate         = Gaps::parse_u16(payload, pos);
  moni.lost_rate    = Gaps::parse_u16(payload, pos);
  u16 tail     = Gaps::parse_u16(payload, pos);
  if (tail != MtbMoniData::TAIL) {
    spdlog::error("No tail signature (0x5555) found for decoding of MtbMoniData!");   
  }
  return moni;
}

std::string MtbMoniData::to_string() const {
  std::string repr = "<MtbMoniData :";
  repr += "\n\t fpga_temp    :" + std::to_string(fpga_temp     );
  repr += "\n\t fpga_vccint  :" + std::to_string(fpga_vccint   );
  repr += "\n\t fpga_vccaux  :" + std::to_string(fpga_vccaux   );
  repr += "\n\t fpga_vccbram :" + std::to_string(fpga_vccbram  );
  repr += "\n\t rate         :" + std::to_string(rate          );
  repr += "\n\t lost_rate    :" + std::to_string(lost_rate     );
  return repr; 
}


TofCmpMoniData::TofCmpMoniData() {
  core1_tmp = 0; 
  core2_tmp = 0; 
  pch_tmp   = 0; 
}

TofCmpMoniData TofCmpMoniData::from_bytestream(const Vec<u8> &payload,
                                               usize &pos) {
  auto moni = TofCmpMoniData();
  u16 head  = Gaps::parse_u16(payload, pos);
  if (head != TofCmpMoniData::HEAD) {
    spdlog::error("No header signature (0xAAAA) found for decoding of TofCmpMoniData!");   
  }
  moni.core1_tmp = Gaps::parse_u8(payload, pos); 
  moni.core2_tmp = Gaps::parse_u8(payload, pos); 
  moni.pch_tmp   = Gaps::parse_u8(payload, pos); 
  u16 tail  = Gaps::parse_u16(payload, pos);
  if (tail != TofCmpMoniData::TAIL) {
    spdlog::error("No tail signature (0x5555) found for decoding of TofCmpMoniData!");   
  }
  return moni;
}

std::string TofCmpMoniData::to_string() const {
  std::string repr = "<TofCmpMoniData : ";
  repr += "\n\t core1_tmp :" + std::to_string(core1_tmp);
  repr += "\n\t core2_tmp :" + std::to_string(core2_tmp);
  repr += "\n\t pch_tmp   :" + std::to_string(pch_tmp) + ">";
  return repr;
}

std::ostream& operator<<(std::ostream& os, const TofCmpMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const RBMoniData& moni){
  os << moni.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const MtbMoniData& moni){
  os << moni.to_string();
  return os;
}

