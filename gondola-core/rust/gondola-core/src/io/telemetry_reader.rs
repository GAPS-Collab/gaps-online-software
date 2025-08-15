//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//!
//! The TelemetryPacketReader allows to read a (file) stream of serialized
//! TelemetryPackets, which are typically the .bin files as generated 
//! by the gcu

use std::io::{
  BufReader,
  Read,
  Seek,
  SeekFrom,
};
use std::fmt;

use regex::Regex;

use crate::io::list_path_contents_sorted;

use crate::reader;

use std::fs::{
  File,
  OpenOptions
};

use crate::packets::{
  TelemetryPacket,
  TelemetryPacketHeader,
  TelemetryPacketType
};

/// Read serialized TelemetryPackets from an existing file
///
/// Read GAPS binary files ("Berkeley binaries)
#[derive(Debug)]
pub struct TelemetryPacketReader {
  /// Reader will emit packets from these files,
  /// if one file is exhausted, it moves on to 
  /// the next file automatically
  pub filenames       : Vec<String>,
  /// The index of the file the reader is 
  /// currently reading
  pub file_idx        : usize,
  file_reader         : BufReader<File>,
  /// Current (byte) position in the file
  cursor              : usize,
  /// Read only packets of type == PacketType
  pub filter          : TelemetryPacketType,
  /// Number of read packets
  n_packs_read        : usize,
  /// Number of skipped packets
  n_packs_skipped     : usize,
  /// Skip the first n packets
  pub skip_ahead      : usize,
  /// Stop reading after n packets
  pub stop_after      : usize,
}

impl fmt::Display for TelemetryPacketReader {
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
    let repr = format!("<TelemetryPacketReader : read {} packets, filter {}, range {},\n files {:?}>", self.n_packs_read, self.filter, range_repr, self.filenames);
    write!(f, "{}", repr)
  }
}

impl TelemetryPacketReader {
  pub fn new(filename_or_directory : String) -> Self {
    let firstfile : String;
    let re = Regex::new(r"RAW(\d{6})_(\d{6})\.bin$").unwrap();
    match list_path_contents_sorted(&filename_or_directory, Some(re)) {
      Err(err) => {
        error!("{} does not seem to be either a valid directory or an existing file! {err}", filename_or_directory);
        panic!("Unable to open files!");
      }
      Ok(files) => {
        firstfile = files[0].clone();
        match OpenOptions::new().create(false).append(false).read(true).open(&firstfile) {
          Err(err) => {
            error!("Unable to open file {firstfile}! {err}");
            panic!("Unable to create reader from {filename_or_directory}!");
          }
          Ok(file) => {
            let packet_reader = Self { 
              filenames       : files,
              file_idx        : 0,
              file_reader     : BufReader::new(file),
              cursor          : 0,
              filter          : TelemetryPacketType::Unknown,
              n_packs_read    : 0,
              skip_ahead      : 0,
              stop_after      : 0,
              n_packs_skipped : 0,
            };
            packet_reader
          }
        }
      }
    }
  } 

  /// Return the next tofpacket in the stream
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  pub fn read_next_item(&mut self) -> Option<TelemetryPacket> {
    // filter::Unknown corresponds to allowing any
    let mut buffer = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          self.prime_next_file()?;
          return self.read_next_item();
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xeb {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            self.prime_next_file()?;
            return self.read_next_item();
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }

        if buffer[0] != 0x90 { 
          continue;
        } else {
          // the 3rd byte is the packet type
          match self.file_reader.read_exact(&mut buffer) {
             Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 1;
            }
          }
          let mut thead = TelemetryPacketHeader::new();
          thead.sync        = 0x90eb;
          thead.packet_type = TelemetryPacketType::from(buffer[0]);
          let ptype    = TelemetryPacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_ts = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_ts) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 4;
              thead.timestamp = u32::from_le_bytes(buffer_ts);
            }
          }
          let mut buffer_counter = [0,0];
          match self.file_reader.read_exact(&mut buffer_counter) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 2;
              thead.counter   = u16::from_le_bytes(buffer_counter);
            }
          }
          let mut buffer_length = [0,0];
          match self.file_reader.read_exact(&mut buffer_length) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
            }
            Ok(_) => {
              self.cursor += 2;
              thead.length    = u16::from_le_bytes(buffer_length);
            }
          }
          let mut buffer_checksum = [0,0];
          match self.file_reader.read_exact(&mut buffer_checksum) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 2;
              thead.checksum    = u16::from_le_bytes(buffer_checksum);
            }
          }
          
          let mut size     = thead.length;
          // This size includes the header
          if (size as usize) < TelemetryPacketHeader::SIZE {
            error!("This packet might be empty or corrupt!");
            return None;
          }
          size -= TelemetryPacketHeader::SIZE as u16;
          if ptype != self.filter && self.filter != TelemetryPacketType::Unknown {
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                self.prime_next_file()?;
                return self.read_next_item();
              }
              Ok(_) => {
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
          // now at this point, we want the packet!
          // except we skip ahead or stop earlier
          if self.skip_ahead > 0 && self.n_packs_skipped < self.skip_ahead {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                self.prime_next_file()?;
                return self.read_next_item();
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
                self.prime_next_file()?;
                return self.read_next_item();
              }
              Ok(_) => {
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
          

          let mut tp = TelemetryPacket::new();
          tp.header  = thead;
          
          //tp.packet_type = ptype;
          //let mut payload = vec![0u8;TelemetryPacketHeader::SIZE];
          //match self.file_reader.read_exact(&mut payload) {
          //  Err(err) => {
          //    debug!("Unable to read from file! {err}");
          //    return None;
          //  }
          //  Ok(_) => {
          //    self.cursor += size as usize;
          //  }
          //}

          let mut payload = vec![0u8;size as usize];
          match self.file_reader.read_exact(&mut payload) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += tp.header.length as usize;
            }
          }

          tp.payload = payload;
          self.n_packs_read += 1;
          return Some(tp);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn
}

impl Default for TelemetryPacketReader {
  fn default() -> Self {
    TelemetryPacketReader::new(String::from(""))
  }
}

reader!(TelemetryPacketReader, TelemetryPacket);
