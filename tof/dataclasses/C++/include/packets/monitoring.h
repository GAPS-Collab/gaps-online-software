#ifndef MONITORINGPACKETS_H_INCLUDED
#define MONITORINGPACKETS_H_INCLUDED

#include <array>
#include "tof_typedefs.h"

/// Monitoring data from the LTBs
///
/// Only one of the RBs per RAT is 
/// connected to the LTB of the RAT
///
/// temperature and threshold 
/// information
///
struct LTBMoniData {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  /// byte size with HEAD + TAIL
  static const u8  SIZE = 25; 
  
  u8                 board_id  ; 
  f32                trenz_temp; 
  f32                ltb_temp  ; 
  std::array<f32, 3> thresh    ; 
  
  LTBMoniData();

  /// Factory function - recreate LTBMoniData from 
  /// byte representation
  static LTBMoniData from_bytestream(const Vec<u8> &stream,
                                     usize &pos);
  
  /// String representatioin for printing
  std::string to_string() const;
};

/// Radoutbaord sensors, covering the RB electronics 
/// as well as the preamps.
/// 
struct RBMoniData {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u8  SIZE = 6;

  u8  board_id           ;  
  /// Rate as recorded by the board itself
  u16 rate               ; 
  f32 tmp_drs            ; 
  f32 tmp_clk            ; 
  f32 tmp_adc            ; 
  /// fpga temperature
  f32 tmp_zynq           ; 
  f32 tmp_lis3mdltr      ; 
  f32 tmp_bm280          ; 
  /// ambient pressure
  f32 pressure           ; 
  /// ambient humidity
  f32 humidity           ; 
  f32 mag_x              ; 
  f32 mag_y              ; 
  f32 mag_z              ; 
  /// total strength of magnetic field
  f32 mag_tot            ; 
  f32 drs_dvdd_voltage   ; 
  f32 drs_dvdd_current   ; 
  f32 drs_dvdd_power     ; 
  f32 p3v3_voltage       ; 
  f32 p3v3_current       ; 
  f32 p3v3_power         ; 
  f32 zynq_voltage       ; 
  f32 zynq_current       ; 
  f32 zynq_power         ; 
  f32 p3v5_voltage       ; 
  f32 p3v5_current       ; 
  f32 p3v5_power         ; 
  f32 adc_dvdd_voltage   ; 
  f32 adc_dvdd_current   ; 
  f32 adc_dvdd_power     ; 
  f32 adc_avdd_voltage   ; 
  f32 adc_avdd_current   ; 
  f32 adc_avdd_power     ; 
  f32 drs_avdd_voltage   ; 
  f32 drs_avdd_current   ; 
  f32 drs_avdd_power     ; 
  f32 n1v5_voltage       ; 
  f32 n1v5_current       ; 
  f32 n1v5_power         ; 

  RBMoniData();

  static RBMoniData from_bytestream(const Vec<u8> &stream,
                                    usize &pos);
  
  /// String representatioin for printing
  std::string to_string() const;
};

std::ostream& operator<<(std::ostream& os, const RBMoniData& moni);

/// MasterTriggerBoard internal sensors
struct MtbMoniData {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u8  SIZE = 6;
  
  f32 fpga_temp    ;
  f32 fpga_vccint  ;
  f32 fpga_vccaux  ;
  f32 fpga_vccbram ;
  u16 rate         ;
  u16 lost_rate    ;
 
  MtbMoniData();

  std::string to_string() const;

  /// extract moni data from payload
  static MtbMoniData from_bytestream(const Vec<u8>& payload,
                                     usize& pos);
};

std::ostream& operator<<(std::ostream& os, const MtbMoniData& moni);

/// System performance and temperature data 
/// of the central tof computer
struct TofCmpMoniData {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u8  SIZE = 6;
  
  u8 core1_tmp ; 
  u8 core2_tmp ; 
  u8 pch_tmp   ; 
 
  TofCmpMoniData(); 
  /// extract moni data from payload
  static TofCmpMoniData from_bytestream(const Vec<u8>& payload,
                                        usize &pos);

  std::string to_string() const;
};

std::ostream& operator<<(std::ostream& os, const TofCmpMoniData& moni);

#endif
