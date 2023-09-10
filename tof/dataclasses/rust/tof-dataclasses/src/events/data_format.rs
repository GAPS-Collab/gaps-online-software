use std::fmt;

/// Data format adds meta information about 
/// the syntax of the data
///
/// Describe the layout of the data in 
/// Memory
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataFormat {
  Default,
  HeaderOnly,
  MemoryView,
  Unknown,
}

impl fmt::Display for DataFormat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<DataFormat: {}>", r)
  }
}

impl DataFormat {
  pub const UNKNOWN               : u8 = 0;
  pub const DEFAULT               : u8 = 10;
  pub const HEADERONLY            : u8 = 20;
  pub const MEMORYVIEW            : u8 = 30;

  pub fn to_u8(&self) -> u8 {
    let result : u8;
    match self {
      DataFormat::Unknown => {
        result = DataFormat::UNKNOWN;
      }
      DataFormat::Default => {
        result = DataFormat::DEFAULT;
      }
      DataFormat::HeaderOnly => {
        result = DataFormat::HEADERONLY;
      }
      DataFormat::MemoryView => {
        result = DataFormat::MEMORYVIEW;
      }
    }
    result
  }
  
  pub fn from_u8(code : &u8) -> Self {
    let mut result = DataFormat::Unknown;
    match *code {
      DataFormat::UNKNOWN => {
        result = DataFormat::Unknown;
      }
      DataFormat::DEFAULT => {
        result = DataFormat::Default;
      }
      DataFormat::HEADERONLY => {
        result = DataFormat::HeaderOnly;
      }
      DataFormat::MEMORYVIEW => {
        result = DataFormat::MemoryView;
      }
      _ => {
        error!("Unknown DataFormat {}!", code);
      }
    }
    result
  }

  /// String representation of the DataFormat
  ///
  /// This is basically the enum type as 
  /// a string.
  pub fn string_repr(&self) -> String { 
    let repr : String;
    match self {
      DataFormat::Unknown => {
        repr = String::from("Unknown");
      }
      DataFormat::Default => {
        repr = String::from("Default");
      }
      DataFormat::HeaderOnly => {
        repr = String::from("HeaderOnly");
      }
      DataFormat::MemoryView => {
        repr = String::from("MemoryView");
      }
    }
    repr
  }
}

#[test]
fn test_data_format() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(DataFormat::UNKNOWN); 
  type_codes.push(DataFormat::DEFAULT); 
  type_codes.push(DataFormat::HEADERONLY); 
  type_codes.push(DataFormat::MEMORYVIEW); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,DataFormat::to_u8(&DataFormat::from_u8(tc)));  
  }
}


