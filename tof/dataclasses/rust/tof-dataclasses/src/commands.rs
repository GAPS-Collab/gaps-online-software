///! Commmands which can be issued
///  to the various components of 
///  the tof system.



pub enum TofCommand {
  ColdRestart  = 10,
  WarmRestart  = 11,
  Calibrate    = 20,
  DataRunStart = 90, 
  DataRunEnd   = 99
}
