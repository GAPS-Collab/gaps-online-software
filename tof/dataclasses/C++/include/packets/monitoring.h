#ifndef MONITORINGPACKETS_H_INCLUDED
#define MONITORINGPACKETS_H_INCLUDED

#include "tof_typedefs.h"

/// Radoutbaord sensors, covering the RB electronics 
/// as well as the preamps.
/// 
class RBMoniData {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u8  SIZE = 6;

  u8  board_id           ; 
  u16 rate               ; 
  f32 tmp_drs            ; 
  f32 tmp_clk            ; 
  f32 tmp_adc            ; 
  f32 tmp_zynq           ; 
  f32 tmp_lis3mdltr      ; 
  f32 tmp_bm280          ; 
  f32 pressure           ; 
  f32 humidity           ; 
  f32 mag_x              ; 
  f32 mag_y              ; 
  f32 mag_z              ; 
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

  /// extract moni data from payload
  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);

};


/// MasterTriggerBoard internal sensors
class MtbMoniData {
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

  /// extract moni data from payload
  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);
};

/// System performance and temperature data 
/// of the central tof computer
class TofCmpMoniData {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u8  SIZE = 6;
  
  u8 core1_tmp ; 
  u8 core2_tmp ; 
  u8 pch_tmp   ; 
 
  TofCmpMoniData(); 
  /// extract moni data from payload
  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);
};

#endif
