#ifndef COMMANDS_H_INCLUDED
#define COMMANDS_H_INCLUDED

enum class TofCommandCode {
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

#endif
