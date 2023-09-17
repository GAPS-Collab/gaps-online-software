/// PacketType identifies the payload in TofPackets
///
/// This needs to be kept in sync with the C++ API
use std::fmt;

/// Types of serializable data structures used
/// throughout the tof system
#[derive(Debug, PartialEq, Clone)]
pub enum PacketType {
  Unknown       , 
  Command       ,
  TofEvent      ,
  Monitor       ,
  MasterTrigger , 
  HeartBeat     ,
  Scalar        ,
  RBHeader      ,
  RBEventPayload,
  RBEvent       ,
  RBEventMemoryView,
  RBMoni     ,
  MonitorTofCmp ,
  MonitorMtb    ,
  RBCalibration ,
}

impl PacketType {
  pub const UNKNOWN           : u8 =  0;
  pub const COMMAND           : u8 = 10;
  pub const RBEVENT           : u8 = 20;
  pub const TOFEVENT          : u8 = 21;
  // not specific enough, deprecated. Use the packet
  // types for monitor packets below
  pub const MONITOR           : u8 = 30;
  pub const HEARTBEAT         : u8 = 40;
  pub const SCALAR            : u8 = 50;
  pub const MT                : u8 = 60;
  pub const RBHEADER          : u8 = 70;
  // monitoring packets
  pub const TOFCMP_MONI       : u8 = 80;
  pub const MTB_MONI          : u8 = 90;
  pub const RB_MONI           : u8 = 100;
  pub const RBEVENTPAYLOAD    : u8 = 110;
  pub const RBEVENTMEMORYVIEW : u8 = 120;
  pub const RBCALIBRATION     : u8 = 130;

  pub fn as_u8(packet_type : &PacketType)   -> u8 {
    match packet_type {
      PacketType::Unknown           => Self::UNKNOWN,
      PacketType::Command           => Self::COMMAND,
      PacketType::RBEvent           => Self::RBEVENT,
      PacketType::RBEventPayload    => Self::RBEVENTPAYLOAD, 
      PacketType::RBHeader          => Self::RBHEADER,
      PacketType::RBEventMemoryView => Self::RBEVENTMEMORYVIEW,
      PacketType::TofEvent          => Self::TOFEVENT,
      PacketType::Monitor           => Self::MONITOR,
      PacketType::HeartBeat         => Self::HEARTBEAT,
      PacketType::MasterTrigger     => Self::MT,
      PacketType::Scalar            => Self::SCALAR,
      PacketType::RBMoni            => Self::RB_MONI,
      PacketType::MonitorTofCmp     => Self::TOFCMP_MONI,
      PacketType::MonitorMtb        => Self::MTB_MONI,
      PacketType::RBCalibration     => Self::RBCALIBRATION,
    }
  }

  pub fn from_u8(value : u8) -> Option<PacketType> {
    match value {
      Self::UNKNOWN           => Some(PacketType::Unknown),  
      Self::COMMAND           => Some(PacketType::Command), 
      Self::RBEVENT           => Some(PacketType::RBEvent), 
      Self::TOFEVENT          => Some(PacketType::TofEvent),
      Self::MONITOR           => Some(PacketType::Monitor), 
      Self::HEARTBEAT         => Some(PacketType::HeartBeat),
      Self::MT                => Some(PacketType::MasterTrigger),
      Self::SCALAR            => Some(PacketType::Scalar),
      Self::RBHEADER          => Some(PacketType::RBHeader),
      Self::RBEVENTPAYLOAD    => Some(PacketType::RBEventPayload),
      Self::RBEVENTMEMORYVIEW => Some(PacketType::RBEventMemoryView),
      Self::RB_MONI           => Some(PacketType::RBMoni),
      Self::MTB_MONI          => Some(PacketType::MonitorMtb),
      Self::TOFCMP_MONI       => Some(PacketType::MonitorTofCmp),
      Self::RBCALIBRATION     => Some(PacketType::RBCalibration),
      _   => None,
    }
  }
}

impl fmt::Display for PacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    
    let repr : String;
    match self {
      PacketType::Unknown           => { repr = String::from("Unknown")     },
      PacketType::Command           => { repr = String::from("Command")     },
      PacketType::RBEvent           => { repr = String::from("RBEvent")     },
      PacketType::RBEventPayload    => { repr = String::from("RBEventPayload") },
      PacketType::RBEventMemoryView => { repr = String::from("RBEventMemoryView") },
      PacketType::TofEvent          => { repr = String::from("TOFEvent")    },
      PacketType::Monitor           => { repr = String::from("Monitor")     },
      PacketType::HeartBeat         => { repr = String::from("HeartBeat")   },
      PacketType::MasterTrigger     => { repr = String::from("MasterTrigger") },
      PacketType::Scalar            => { repr = String::from("Scalar")      },
      PacketType::RBHeader          => { repr = String::from("RBHeadher")    },
      PacketType::RBMoni            => { repr = String::from("RBMoni")     },
      PacketType::MonitorTofCmp     => { repr = String::from("TOFCMPMoni") },
      PacketType::MonitorMtb        => { repr = String::from("MTBMoni")    },
      PacketType::RBCalibration     => { repr = String::from("RBCalibration")},
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
  type_codes.push(PacketType::SCALAR           ); 
  type_codes.push(PacketType::MT               ); 
  type_codes.push(PacketType::RBHEADER         ); 
  type_codes.push(PacketType::TOFCMP_MONI      ); 
  type_codes.push(PacketType::MTB_MONI         ); 
  type_codes.push(PacketType::RB_MONI          );
  type_codes.push(PacketType::RBEVENTPAYLOAD   );
  type_codes.push(PacketType::RBEVENTMEMORYVIEW);
  type_codes.push(PacketType::RBCALIBRATION    );
  for tc in type_codes.iter() {
    assert_eq!(*tc,PacketType::as_u8(&PacketType::from_u8(*tc).unwrap()));  
  }
}

