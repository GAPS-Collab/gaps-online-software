/// PacketType identifies the payload in TofPackets
///
/// This needs to be kept in sync with the C++ API
use std::fmt;

/// Types of serializable data structures used
/// throughout the tof system
#[derive(Debug, PartialEq, Clone)]
pub enum PacketType {
  Unknown       , 
  TofEvent      ,
  Monitor       ,
  MasterTrigger , 
  HeartBeat     ,
  RBEventHeader ,
  RBEvent       ,
  RBEventMemoryView,
  TofCommand    ,
  RBCommand     ,
  RBMoni        ,
  MonitorTofCmp ,
  MonitorMtb    ,
  RBCalibration ,
}

impl PacketType {
  pub const UNKNOWN           : u8 =  0;
  pub const RBEVENT           : u8 = 20;
  pub const TOFEVENT          : u8 = 21;
  // not specific enough, deprecated. Use the packet
  // types for monitor packets below
  pub const MONITOR           : u8 = 30;
  pub const HEARTBEAT         : u8 = 40;
  pub const MT                : u8 = 60;
  pub const RBEVENTHEADER     : u8 = 70;
  // monitoring packets
  pub const TOFCMP_MONI       : u8 = 80;
  pub const MTB_MONI          : u8 = 90;
  pub const RB_MONI           : u8 = 100;
  pub const RBEVENTMEMORYVIEW : u8 = 120;
  pub const RBCALIBRATION     : u8 = 130;
  pub const TOFCOMMAND        : u8 = 140;
  pub const RBCOMMAND         : u8 = 150;

  pub fn as_u8(packet_type : &PacketType)   -> u8 {
    match packet_type {
      PacketType::Unknown           => Self::UNKNOWN,
      PacketType::RBEvent           => Self::RBEVENT,
      PacketType::RBEventHeader          => Self::RBEVENTHEADER,
      PacketType::RBEventMemoryView => Self::RBEVENTMEMORYVIEW,
      PacketType::TofEvent          => Self::TOFEVENT,
      PacketType::Monitor           => Self::MONITOR,
      PacketType::HeartBeat         => Self::HEARTBEAT,
      PacketType::MasterTrigger     => Self::MT,
      PacketType::RBMoni            => Self::RB_MONI,
      PacketType::MonitorTofCmp     => Self::TOFCMP_MONI,
      PacketType::MonitorMtb        => Self::MTB_MONI,
      PacketType::RBCalibration     => Self::RBCALIBRATION,
      PacketType::RBCommand         => Self::RBCOMMAND,
      PacketType::TofCommand        => Self::TOFCOMMAND,
    }
  }

  pub fn from_u8(value : u8) -> Option<PacketType> {
    match value {
      Self::UNKNOWN           => Some(PacketType::Unknown),  
      Self::RBEVENT           => Some(PacketType::RBEvent), 
      Self::TOFEVENT          => Some(PacketType::TofEvent),
      Self::MONITOR           => Some(PacketType::Monitor), 
      Self::HEARTBEAT         => Some(PacketType::HeartBeat),
      Self::MT                => Some(PacketType::MasterTrigger),
      Self::RBEVENTHEADER     => Some(PacketType::RBEventHeader),
      Self::RBEVENTMEMORYVIEW => Some(PacketType::RBEventMemoryView),
      Self::RB_MONI           => Some(PacketType::RBMoni),
      Self::MTB_MONI          => Some(PacketType::MonitorMtb),
      Self::TOFCMP_MONI       => Some(PacketType::MonitorTofCmp),
      Self::RBCALIBRATION     => Some(PacketType::RBCalibration),
      Self::TOFCOMMAND        => Some(PacketType::TofCommand),
      Self::RBCOMMAND         => Some(PacketType::RBCommand),
      _   => None,
    }
  }
}

impl fmt::Display for PacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    
    let repr : String;
    match self {
      PacketType::Unknown           => { repr = String::from("Unknown")     },
      PacketType::RBEvent           => { repr = String::from("RBEvent")     },
      PacketType::RBEventMemoryView => { repr = String::from("RBEventMemoryView") },
      PacketType::TofEvent          => { repr = String::from("TOFEvent")    },
      PacketType::Monitor           => { repr = String::from("Monitor")     },
      PacketType::HeartBeat         => { repr = String::from("HeartBeat")   },
      PacketType::MasterTrigger     => { repr = String::from("MasterTrigger") },
      PacketType::RBEventHeader          => { repr = String::from("RBEventHeader")    },
      PacketType::RBMoni            => { repr = String::from("RBMoni")     },
      PacketType::MonitorTofCmp     => { repr = String::from("TOFCMPMoni") },
      PacketType::MonitorMtb        => { repr = String::from("MTBMoni")    },
      PacketType::RBCalibration     => { repr = String::from("RBCalibration")},
      PacketType::TofCommand        => { repr = String::from("TofCommand")},
      PacketType::RBCommand         => { repr = String::from("RBCommand")},
    }
    write!(f, "<PacketType {}>", repr)
  }
}

#[test]
fn test_packet_types() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(PacketType::UNKNOWN          ); 
  type_codes.push(PacketType::COMMAND          ); 
  type_codes.push(PacketType::RBEVENT          ); 
  type_codes.push(PacketType::TOFEVENT         ); 
  type_codes.push(PacketType::MONITOR          ); 
  type_codes.push(PacketType::HEARTBEAT        ); 
  type_codes.push(PacketType::MT               ); 
  type_codes.push(PacketType::RBEVENTHEADER    ); 
  type_codes.push(PacketType::TOFCMP_MONI      ); 
  type_codes.push(PacketType::MTB_MONI         ); 
  type_codes.push(PacketType::RB_MONI          );
  type_codes.push(PacketType::RBEVENTMEMORYVIEW);
  type_codes.push(PacketType::RBCALIBRATION    );
  type_codes.push(PacketType::RBCOMMAND        );
  type_codes.push(PacketType::TOFCOMMAND       );
  for tc in type_codes.iter() {
    assert_eq!(*tc,PacketType::as_u8(&PacketType::from_u8(*tc).unwrap()));  
  }
}

