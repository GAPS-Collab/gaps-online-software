//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//!
//! The TofPacketReader allows to read a (file) stream of serialized
//! TofPackets
use std::io::{
  BufReader,
  Read,
  Seek,
  SeekFrom,
};

use crate::packets::{
  TofPacketType,
  TofPacket
};
use crate::io::list_path_contents_sorted;
use crate::io::parsers::{
  parse_u16,
  parse_u32
};

use std::fs::{
  File,
  OpenOptions
};

use std::fmt;
use crate::reader;

/// Read serialized TofPackets from an existing file or directory
///
/// This can read the "TOF stream" files, typically suffixed with .tof.gaps
/// These files are typically written by a TofPacketReader instance, e.g. as 
/// on the TOF flight computer
#[derive(Debug)]
pub struct TofPacketReader {
  /// Read from this file
  pub filenames       : Vec<String>,
  file_reader         : BufReader<File>,
  /// Current (byte) position in the file
  cursor              : usize,
  /// Read only packets of type == PacketType
  pub filter          : TofPacketType,
  /// Number of read packets
  n_packs_read        : usize,
  /// Number of skipped packets
  n_packs_skipped     : usize,
  /// Skip the first n packets
  pub skip_ahead      : usize,
  /// Stop reading after n packets
  pub stop_after      : usize,
  /// The index of the current file in the internal "filenames" vector.
  pub file_idx        : usize,
}

impl TofPacketReader {
  
  /// Setup a new Reader, allowing the argument to be either the name of a single file or 
  /// the name of a directory
  pub fn new(filename_or_directory : &str) -> Self {
    let firstfile : String;
    match list_path_contents_sorted(&filename_or_directory, None) {
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
              file_reader     : BufReader::new(file),
              cursor          : 0,
              filter          : TofPacketType::Unknown,
              n_packs_read    : 0,
              skip_ahead      : 0,
              stop_after      : 0,
              n_packs_skipped : 0,
              file_idx        : 0,
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
  ///
  /// If a filter is set, only packets of type as set 
  /// in the filter will be read, all others will be 
  /// ignored
  pub fn read_next_item(&mut self) -> Option<TofPacket> {
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
      if buffer[0] != 0xAA {
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

        if buffer[0] != 0xAA { 
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
          let ptype    = TofPacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            } 
            Ok(_) => {
              self.cursor += 4;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u32(&vec_data, &mut 0);
          if ptype != self.filter && self.filter != TofPacketType::Unknown {
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

          let mut tp = TofPacket::new();
          tp.packet_type = ptype;
          let mut payload = vec![0u8;size as usize];

          match self.file_reader.read_exact(&mut payload) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item(); 
            }
            Ok(_) => {
              self.cursor += size as usize;
            }
          }
          tp.payload = payload;
          // we don't filter, so we like this packet
          let mut tail = vec![0u8; 2];
          match self.file_reader.read_exact(&mut tail) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item(); 
            }
            Ok(_) => {
              self.cursor += 2;
            }
          }
          let tail = parse_u16(&tail,&mut 0);
          if tail != TofPacket::TAIL {
            debug!("TofPacket TAIL signature wrong!");
            return None;
          }
          self.n_packs_read += 1;
          return Some(tp);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn
}

impl fmt::Display for TofPacketReader {
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
    let repr = format!("<TofPacketReader :read {} packets, filter {}, range {}\n files {:?}>", self.n_packs_read, self.filter, range_repr, self.filenames);
    write!(f, "{}", repr)
  }
}

reader!(TofPacketReader,TofPacket);
