// This file is part of gaps-online-software and published 
// under the GPLv3 license

pub mod tof_packet_type;
pub mod tof_packet;
pub mod telemetry_packet_type;
pub mod telemetry_packet_header; 
pub mod telemetry_packet;
pub mod tracker_header;
pub use tracker_header::TrackerHeader;
pub mod bfsw_ack_packet;
pub use bfsw_ack_packet::AckBfsw;
pub mod gps_packet;
pub use gps_packet::GPSPacket;
pub mod tracker;
pub use tracker::{
  TrackerEventIDEchoPacket,
  TrackerTempLeakPacket,
  TrackerDAQTempPacket,
  TrackerDAQHSKPacket
};
pub mod magnetometer;
pub use magnetometer::MagnetoMeter;


// public exports to reduce the Matroshka effect a little
pub use telemetry_packet_type::TelemetryPacketType;
pub use telemetry_packet::TelemetryPacket;
pub use telemetry_packet_header::TelemetryPacketHeader;
pub use tof_packet_type::TofPacketType;
pub use tof_packet::TofPacket;

use crate::io::serialization::Serialization;

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

/// Recreate 48bit timestamp from u32 and u16
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn make_systime(lower : u32, upper : u16) -> u64 {
  (upper as u64) << 32 | lower as u64
}


/// Can be wrapped within a TofPacket. To do, we just have
/// to define a packet type
pub trait TofPackable {
  const TOF_PACKET_TYPE     : TofPacketType;
  // provide an alternative TofPacketType to retrieve the 
  // packet from without failing
  const TOF_PACKET_TYPE_ALT : TofPacketType = TofPacketType::Unknown;

  /// Wrap myself in a TofPacket
  fn pack(&self) -> TofPacket 
    where Self: Serialization {
    let mut tp     = TofPacket::new();
    tp.payload     = self.to_bytestream();
    tp.packet_type = Self::TOF_PACKET_TYPE;
    tp
  }
}

/// Can be wrapped within a TofPacket. To do, we just have
/// to define a packet type
pub trait TelemetryPackable {
  /// packet type for any kind of telemetry packet which is NOT an 
  /// event
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::Unknown;
  /// TelemetryEvents can "occupy" several packet types, e.g. GapsTrigger, Boring, etc
  const TEL_PACKET_TYPES_EVENT : [TelemetryPacketType;4] = [
    TelemetryPacketType::NoGapsTriggerEvent,
    TelemetryPacketType::BoringEvent,
    TelemetryPacketType::InterestingEvent,
    TelemetryPacketType::NoTofDataEvent];
}

