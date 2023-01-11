#ifndef COMMANDPACKET_H_INCLUDED
#define COMMANDPACKET_H_INCLUDED

#include "TofTypeDefs.h"


/*******************************************
 * These are the commands which can be send
 * to the Tof in general and/or the 
 * individual RB.
 *
 * We set individual command codes
 *
 */
enum class TofCommand {
  // power class - 1
  PowerOn               = 11,
  PowerOff              = 10,
  PowerCycle            = 12,
  // setup class - 2
  RBSetup               = 20,
  SetThresholds         = 21,
  SetMtConfig           = 22,
  // run class - 3
  StartValidationRun    = 32,
  DataRunStart          = 31,
  DataRunEnd            = 30,
  // request class -4
  RequestWaveforms      = 41,
  RequestEvent          = 42,
  RequestMoni           = 43,
  // calibration class 5
  VoltageCalibration    = 51,
  TimingCalibration     = 52,
  CreateCalibrationFile = 53,
  Unknown
};


/*********************************************
 * This packet holds all kinds of commands
 * which can be sent to the tof computer/RB
 *
 * The CommandPacket has the following structur
 * 
 * The package layout in binary is like this
 * HEAD        : u16 = 0xAAAA
 * CommnadClas : u8
 * DATA        : u32
 * TAIL        : u16 = 0x5555
 *
 *
 */
struct CommandPacket {
  u16 head = 0xAAAA;
  u16 tail = 0x5555;
  // every command packet is 9 bytes
  u16 p_length_fixed = 9;
  TofCommand command;
  u32 value;

  CommandPacket(const TofCommand &cmd, const u32 value);
  vec_u8 to_bytestream();

  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);

};


#endif
