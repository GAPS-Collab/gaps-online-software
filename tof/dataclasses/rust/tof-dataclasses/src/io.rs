//! Input/Output 
//!
//! * Read files into memory
//!   
//!
//!
//!
//!

use std::path::Path;
use std::fs::{self,File};
use std::io::{self,Read};


/// Read an entire file into memory
///
/// Represents the contents of a file 
/// as a byte vector
/// 
/// # Arguments:
///
/// * filename : full path to the file to be read
pub fn read_file(filename: &Path) -> io::Result<Vec<u8>> {
  info!("Reading file {}", filename.display());
  let mut f = File::open(&filename)?;
  let metadata = fs::metadata(&filename)?;
  let mut buffer = vec![0; metadata.len() as usize];
  info!("Read {} bytes from {}", buffer.len(), filename.display());
  // read_exact if the amount is not specified
  f.read_exact(&mut buffer)?;
  Ok(buffer)
}