///
/// Monitoring 
/// thread + classes
///
///
///
///

use std::mem::size_of;

use crate::constants::MAX_NBOARDS;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Encoding<T> {
  IsNativeU64(u64),
  IsNativeU32(u32),
  IsNativeU16(u16),
  IsNativeU8(u8),
  IsF64(f64),
  IsF32(f32),
  Unknown(T)
}

#[derive(Debug, Clone)]
struct MonitoringPacket<T> {
  pub label      : String,
  pub label_size : u8,
  pub value      : T,
  pub value_size : usize
}

#[derive(Debug, Copy, Clone)]
struct TriggerInfo {
  pub mt_rate          : u32,
  pub n_events         : u64,
  pub lost_mt_triggers : u64,
  pub rb_rate          : [u32; MAX_NBOARDS]
}


impl<T> MonitoringPacket<T> {
  pub fn new(label : String, value : T) -> MonitoringPacket<T> {
    // we don't like long labels
    let label_len = label.len();
    if label_len > 255 {
      panic!("The label is too long and has more than 255 characters! label {}, Please restrict yourself to shorter labels", label); 
    } 

    MonitoringPacket::<T> {
      label      : label,
      label_size : label_len as u8,
      value      : value,
      value_size : size_of::<T>(),
    }

  }

}


