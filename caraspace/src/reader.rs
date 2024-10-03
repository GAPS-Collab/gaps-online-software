//! Reader
//!

use std::fmt;

use std::fs::{
    File,
    OpenOptions
};
use std::io;
use std::io::{
    BufReader,
    Seek,
    SeekFrom,
    Read,
};

use crate::frame::CRFrame;
use crate::parsers::*;
use crate::serialization::CRSerializeable;

/// Read binaries written through the caraspace system
///
/// The file needs to contain subsequent CRFrames.
#[derive(Debug)]
pub struct CRReader {
  /// Read from this file
  pub filename        : String,
  file_reader         : BufReader<File>,
  /// Current (byte) position in the file
  cursor              : usize,
  /// Number of read packets
  n_packs_read        : usize,
  /// Number of skipped packets
  n_packs_skipped     : usize,
  /// Number of deserialization errors occured
  /// since the beginning of the file
  pub n_errors        : usize,
  /// Skip the first n packets
  pub skip_ahead      : usize,
  /// Stop reading after n packets
  pub stop_after      : usize,
}

impl fmt::Display for CRReader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut range_repr = String::from("");
    if self.skip_ahead > 0 {
      range_repr += &(format!("({}", self.skip_ahead));
    } else {
      range_repr += "(";
    }
    if self.stop_after > 0 {
      range_repr += &(format!("..{})", self.stop_after));
    } else {
      range_repr += "..)";
    }
    let repr = format!("<CRReader : file {}, read {} packets, {} errors, range {}>", self.filename, self.n_packs_read, self.n_errors, range_repr);
    write!(f, "{}", repr)
  }
}

impl CRReader {

  pub fn new(filename : String) -> CRReader {
    let fname_c = filename.clone();
    let file = OpenOptions::new().create(false).append(false).read(true).open(fname_c).expect("Unable to open file {filename}");
    let packet_reader = Self { 
      filename,
      file_reader     : BufReader::new(file),
      cursor          : 0,
      n_packs_read    : 0,
      n_errors        : 0,
      skip_ahead      : 0,
      stop_after      : 0,
      n_packs_skipped : 0,
    };
    packet_reader
  } 

  /// Preview the number of frames in this reader
  pub fn get_n_frames(&mut self) -> usize {
    let mut nframes = 0usize;
    let mut buffer  = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          //return None;
          break;
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xAA {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            //return None;
            break;
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }

        if buffer[0] != 0xAA { 
          continue;
        } else {
          // the 3rd byte is the packet type
          match self.file_reader.read_exact(&mut buffer) {
             Err(err) => {
              debug!("Unable to read from file! {err}");
              break;
            }
            Ok(_) => {
              self.cursor += 1;
            }
          }
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              error!("Unable to read from file! {err}");
              break;
            }
            Ok(_) => {
              self.cursor += 8;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u64(&vec_data, &mut 0);
          match self.file_reader.seek(SeekFrom::Current(size as i64)) {
            Err(err) => {
              debug!("Unable to read more data! {err}");
              break; 
            }
            Ok(_) => {
              self.cursor += size as usize;
              nframes += 1;
              // and then we add the packet type to the 
              // hashmap
            }
          }
        }
      } // if no 0xAA found
    } // end loop
    // FIXME
    let _ = self.rewind();
    nframes
  } // end fn

  pub fn rewind(&mut self) -> io::Result<()> {
    self.file_reader.rewind()?;
    self.cursor = 0;
    Ok(())
  }

  /// Return the next tofpacket in the stream
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  pub fn get_next_packet(&mut self) -> Option<CRFrame> {
    // filter::Unknown corresponds to allowing any

    let mut buffer = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          return None;
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xAA {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            return None;
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }

        if buffer[0] != 0xAA { 
          continue;
        } else {
          // the 3rd byte is the packet type
          //match self.file_reader.read_exact(&mut buffer) {
          //   Err(err) => {
          //    debug!("Unable to read from file! {err}");
          //    return None;
          //  }
          //  Ok(_) => {
          //    self.cursor += 1;
          //  }
          //}
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
            }
            Ok(_) => {
              self.cursor += 8;
            }
          }
          
          let vec_data = buffer_psize.to_vec();
          //println!("vec_data {:?}", vec_data);
          let size     = parse_u64(&vec_data, &mut 0);
          //println!("Will read {size} bytes for payload!");
          // now at this point, we want the packet!
          // except we skip ahead or stop earlier
          if self.skip_ahead > 0 && self.n_packs_skipped < self.skip_ahead {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                return None; 
              }
              Ok(_) => {
                self.n_packs_skipped += 1;
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
          if self.stop_after > 0 && self.n_packs_read >= self.stop_after {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                return None; 
              }
              Ok(_) => {
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want

          }

          let mut frame = CRFrame::new();
          let mut payload = vec![0u8;size as usize];

          match self.file_reader.read_exact(&mut payload) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
            }
            Ok(_) => {
              self.cursor += size as usize;
            }
          }
          let mut in_frame_pos = 0usize;
          frame.index = CRFrame::parse_index(&payload, &mut in_frame_pos);
          frame.bytestorage = payload[in_frame_pos..].to_vec();

          //tp.payload = payload;
          // we don't filter, so we like this packet
          let mut tail = vec![0u8; 2];
          match self.file_reader.read_exact(&mut tail) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
            }
            Ok(_) => {
              self.cursor += 2;
            }
          }
          let tail = parse_u16(&tail,&mut 0);
          if tail != CRFrame::CRTAIL {
            debug!("CRFrame TAIL signature wrong!");
            return None;
          }
          self.n_packs_read += 1;
          return Some(frame);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn
}

impl Default for CRReader {
  fn default() -> Self {
    CRReader::new(String::from(""))
  }
}

impl Iterator for CRReader {
  type Item = CRFrame;
  
  fn next(&mut self) -> Option<Self::Item> {
    self.get_next_packet()
  }
}



