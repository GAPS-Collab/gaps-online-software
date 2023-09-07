use std::fmt;

/// A generic data type
///
/// Describe the purpose of the data. This
/// is the semantics behind it. 
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataType {
  VoltageCalibration,
  TimingCalibration,
  Noi,
  Physics,
  Unknown,
}

impl fmt::Display for DataType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<DataType: {}>", r)
  }
}

impl DataType {
  pub const UNKNOWN               : u8 = 0;
  pub const VOLTAGECALIBRATION    : u8 = 10;
  pub const TIMINGCALIBRATION     : u8 = 20;
  pub const NOI                   : u8 = 30;
  pub const PHYSICS               : u8 = 40;

  pub fn to_u8(&self) -> u8 {
    let result : u8;
    match self {
      DataType::Unknown => {
        result = DataType::UNKNOWN;
      }
      DataType::VoltageCalibration => {
        result = DataType::VOLTAGECALIBRATION;
      }
      DataType::TimingCalibration => {
        result = DataType::TIMINGCALIBRATION;
      }
      DataType::Noi => {
        result = DataType::NOI;
      }
      DataType::Physics => {
        result = DataType::PHYSICS;
      }
    }
    result
  }
  
  pub fn from_u8(code : &u8) -> Self {
    let mut result = DataType::Unknown;
    match *code {
      DataType::UNKNOWN => {
        result = DataType::Unknown;
      }
      DataType::VOLTAGECALIBRATION => {
        result = DataType::VoltageCalibration;
      }
      DataType::TIMINGCALIBRATION => {
        result = DataType::TimingCalibration;
      }
      DataType::NOI => {
        result = DataType::Noi;
      }
      DataType::PHYSICS => {
        result = DataType::Physics;
      }
      _ => {
        error!("Unknown DataType {}!", code);
      }
    }
    result
  }

  /// String representation of the DataType
  ///
  /// This is basically the enum type as 
  /// a string.
  pub fn string_repr(&self) -> String { 
    let repr : String;
    match self {
      DataType::Unknown => {
        repr = String::from("Unknown");
      }
      DataType::VoltageCalibration => {
        repr = String::from("VoltageCalibration");
      }
      DataType::TimingCalibration => {
        repr = String::from("TimingCalibration");
      }
      DataType::Noi => {
        repr = String::from("Noi");
      }
      DataType::Physics => {
        repr = String::from("Physics");
      }
    }
    repr
  }
}

#[test]
fn test_data_type() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(DataType::UNKNOWN); 
  type_codes.push(DataType::VOLTAGECALIBRATION); 
  type_codes.push(DataType::TIMINGCALIBRATION); 
  type_codes.push(DataType::NOI); 
  type_codes.push(DataType::PHYSICS); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,DataType::to_u8(&DataType::from_u8(tc)));  
  }
}

