use std::fmt;

extern crate serde;
extern crate serde_json;

/// Data format adds meta information about 
/// the syntax of the data
///
/// Describe the layout of the data in 
/// Memory
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum DataFormat {
  Default     = 0u8,
  HeaderOnly  = 10u8,
  MemoryView  = 20u8,
  Unknown     = 30u8,
}

impl fmt::Display for DataFormat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this DataFormat"));
    write!(f, "<DataFormat: {}>", r)
  }
}

impl TryFrom<u8> for DataFormat {
  type Error = &'static str;

  // I am not sure about this hard coding, but the code
  //  looks nicer - Paolo
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0u8  => Ok(DataFormat::Default),
      10u8 => Ok(DataFormat::HeaderOnly),
      20u8 => Ok(DataFormat::MemoryView),
      30u8 => Ok(DataFormat::Unknown),
      _    => Err("I am not sure how to convert this value!")
    }
  }
}

#[test]
fn test_data_format() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(DataFormat::Unknown as u8); 
  type_codes.push(DataFormat::Default as u8); 
  type_codes.push(DataFormat::HeaderOnly as u8); 
  type_codes.push(DataFormat::MemoryView as u8); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,DataFormat::try_from(*tc).unwrap() as u8);
  }
}


