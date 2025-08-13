//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

pub mod tof_packet_type;
pub mod tof_packet;
pub mod telemetry_packet_type;
pub mod telemetry_packet_header; 
pub mod telemetry_packet;

// public exports to reduce the Matroshka effect a little
pub use telemetry_packet_type::TelemetryPacketType;
pub use telemetry_packet::TelemetryPacket;
pub use telemetry_packet_header::TelemetryPacketHeader;
pub use tof_packet_type::TofPacketType;
pub use tof_packet::TofPacket;

use crate::io::serialization::Serialization;

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


