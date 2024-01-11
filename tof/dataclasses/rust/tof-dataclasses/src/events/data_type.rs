use std::fmt;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

/// A generic data type
///
/// Describe the purpose of the data. This
/// is the semantics behind it.
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum DataType {
  Unknown            = 0u8,
  VoltageCalibration = 10u8,
  TimingCalibration  = 20u8,
  Noi                = 30u8,
  Physics            = 40u8,
  RBTriggerPeriodic  = 50u8,
  RBTriggerPoisson   = 60u8,
  MTBTriggerPoisson  = 70u8,
  // future extension for different trigger settings!
}

impl fmt::Display for DataType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this DataType"));
    write!(f, "<DataType: {}>", r)
  }
}

impl From<u8> for DataType {
  fn from(value: u8) -> Self {
    match value {
      0u8  => DataType::Unknown,
      10u8 => DataType::VoltageCalibration,
      20u8 => DataType::TimingCalibration,
      30u8 => DataType::Noi,
      40u8 => DataType::Physics,
      50u8 => DataType::RBTriggerPeriodic,
      60u8 => DataType::RBTriggerPoisson,
      70u8 => DataType::MTBTriggerPoisson,
      _    => DataType::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for DataType {
  
  fn from_random() -> Self {
    let choices = [
      DataType::Unknown,
      DataType::VoltageCalibration,
      DataType::TimingCalibration,
      DataType::Noi,
      DataType::Physics,
      DataType::RBTriggerPeriodic,
      DataType::RBTriggerPoisson,
      DataType::MTBTriggerPoisson
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

#[test]
fn test_data_type() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(DataType::Unknown as u8); 
  type_codes.push(DataType::VoltageCalibration as u8); 
  type_codes.push(DataType::TimingCalibration as u8); 
  type_codes.push(DataType::Noi as u8); 
  type_codes.push(DataType::Physics as u8); 
  type_codes.push(DataType::MTBTriggerPoisson as u8); 
  type_codes.push(DataType::RBTriggerPeriodic as u8); 
  type_codes.push(DataType::RBTriggerPoisson as u8); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,DataType::try_from(*tc).unwrap() as u8);
  }
}

