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
  VoltageCalibration = 0u8,
  TimingCalibration  = 10u8,
  Noi                = 20u8,
  Physics            = 30u8,
  RBTriggerPeriodic  = 40u8,
  RBTriggerPoisson   = 50u8,
  MTBTriggerPoisson  = 60u8,
  // future extension for different trigger settings!
  Unknown            = 70u8,
}

impl fmt::Display for DataType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this DataType"));
    write!(f, "<DataType: {}>", r)
  }
}

impl TryFrom<u8> for DataType {
  type Error = &'static str;

  // I am not sure about this hard coding, but the code
  //  looks nicer - Paolo
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0u8  => Ok(DataType::VoltageCalibration),
      10u8 => Ok(DataType::TimingCalibration),
      20u8 => Ok(DataType::Noi),
      30u8 => Ok(DataType::Physics),
      40u8 => Ok(DataType::RBTriggerPeriodic),
      50u8 => Ok(DataType::RBTriggerPoisson),
      60u8 => Ok(DataType::MTBTriggerPoisson),
      70u8 => Ok(DataType::Unknown),
      _    => Err("I am not sure how to convert this value!")
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for DataType {
  
  fn from_random() -> Self {
    let choices = [
      DataType::VoltageCalibration,
      DataType::TimingCalibration,
      DataType::Noi,
      DataType::Physics,
      DataType::RBTriggerPeriodic,
      DataType::RBTriggerPoisson,
      DataType::MTBTriggerPoisson,
      DataType::Unknown
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

