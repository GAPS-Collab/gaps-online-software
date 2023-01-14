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
 * THE COMMAND CODES MUST BE THE 
 * "OFFICAL" COMMAND CODES!
 * Please see the documentation!
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
  UnspoolEventCache     = 44,
  StreamAnyEvent        = 45, 
  // calibration class 5
  VoltageCalibration    = 51,
  TimingCalibration     = 52,
  CreateCalibrationFile = 53,
  Unknown
};

enum class TofResponse {
  Success            = 1,
  GeneralFailure     = 2,
  EventNotReady      = 3,
  SerializationIssue = 4,
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
  const u16 head = 0xAAAA;
  const u16 tail = 0x5555;
  // every command packet is 9 bytes
  u16 p_length_fixed = 9;
  TofCommand command;
  u32 value;

  CommandPacket(const TofCommand &cmd, const u32 value);
  vec_u8 to_bytestream();

  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);

};


/**************************************
 * Each Command will trigger the system
 * to send a response
 *
 * The response consists of a class 
 * (see the above enum) + a specific,
 * 32bit response code.
 *
 *************************/

struct ResponsePacket {
  // these are the specific response code
  static const u32 RESP_ERR_LEVEL_NOPROBLEM	        = 4000;
  static const u32 RESP_ERR_LEVEL_MEDIUM            = 4010;
  static const u32 RESP_ERR_LEVEL_CRITICAL	        = 4030; 
  static const u32 RESP_ERR_LEVEL_MISSION_CRITICAL  = 4040; 
  static const u32 RESP_ERR_LEVEL_RUN_FOOL_RUN	    = 99999; 
  static const u32 RESP_ERR_LEVEL_SEVERE            = 4020; 
  static const u32 RESP_ERR_NORUNACTIVE	            = 501; 
  static const u32 RESP_ERR_NOTIMPLEMENTED          = 404; 
  static const u32 RESP_ERR_RUNACTIVE	            = 502; 
  static const u32 RESP_ERR_UNEXECUTABLE	        = 500; 
  static const u32 RESP_SUCC_FINGERS_CROSSED	    = 200; 
  
  u16 head = 0xAAAA;
  u16 tail = 0x5555;
  // every command packet is 9 bytes
  u16 p_length_fixed = 9;
  TofResponse response;
  u32 value;

  ResponsePacket(const TofResponse &resp, const u32 value);
  vec_u8 to_bytestream() const;

  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);

  //! Get a string represntation of the response codes
  std::string translate_response_code(u32 code) const;
};

#endif
