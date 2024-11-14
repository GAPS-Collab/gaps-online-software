use std::fmt;
use std::fs::File;
use std::io;
use std::io::SeekFrom;
use std::io::Seek;
use std::io::BufReader;
use std::path::Path;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;

use log::{
  debug,
  error
};

use tof_dataclasses::io::read_file;
use tof_dataclasses::serialization::{
  search_for_u16,
  Serialization,
  //parse_u16,
  parse_u32,
};

use crate::packets::{
  TelemetryHeader,
  TelemetryPacket,
  MergedEvent,
  TrackerPacket,
  GapsEvent,
};
use tof_dataclasses::packets::{
  TofPacket,
  PacketType,
};
use tof_dataclasses::events::TofEventSummary;
use crate::packets::TelemetryPacketType;

/// Extract all merged events from a file and ignore all others
pub fn get_gaps_events(filename : String) -> Vec<GapsEvent> {
  let mut events = Vec::<GapsEvent>::new();
  let stream = read_file(Path::new(&filename)).expect("Unable to open input file!");
  let mut pos : usize = 0;
  //let mut npackets : usize = 0;
  let mut packet_types = Vec::<u8>::new();
  loop {
    match TelemetryHeader::from_bytestream(&stream, &mut pos) {
      Err(err) => {
        println!("Can not decode telemtry header! {err}");
        //for k in pos - 5 .. pos + 5 {
        //  println!("{}",stream[k]);
        //}
        match search_for_u16(0x90eb, &stream, pos) {
          Err(err) => {
            println!("Unable to find next header! {err}");
            break;
          }
          Ok(head_pos) => {
            pos = head_pos;
          }
        }
      }
      Ok(header) => {
        println!("HEADER {}", header);
        //for k in pos - 10 .. pos + 10 {
        //  println!("{}",stream[k]);
        //}
        if header.ptype == 80 {
          match TrackerPacket::from_bytestream(&stream, &mut pos) {
            Err(err) => {
              //for k in pos - 5 .. pos + 5 {
              //  println!("{}",stream[k]);
              //}
              println!("Unable to decode TrackerPacket! {err}");
            }
            Ok(mut tp) => {
              tp.telemetry_header = header;
              println!("{}", tp);
            }
          }
        }
        if header.ptype == 90 {
          match MergedEvent::from_bytestream(&stream, &mut pos) {
            Err(err) => {
              println!("Unable to decode MergedEvent! {err}");
            }
            Ok(mut me) => {
              me.header  = header;
              let mut g_event = GapsEvent::new();
              //println!("Event ID  : {}", me.event_id);
              //println!("Tof bytes : {:?}", me.tof_data);
              //println!("len tof bytes : {}", me.tof_data.len());
              match TofPacket::from_bytestream(&me.tof_data, &mut 0) {
                Err(err) => {
                  println!("Can't unpack TofPacket! {err}");
                }
                Ok(tp) => {
                  println!("{}", tp);
                  if tp.packet_type == PacketType::TofEventSummary {
                    match TofEventSummary::from_tofpacket(&tp) {
                      Err(err) => println!("Can't unpack TofEventSummary! {err}"),
                      Ok(ts) => {
                        println!("{}", ts);
                        g_event.tof = ts;
                      }
                    }
                  }
                }
              }
              g_event.tracker = me.tracker_events;
              events.push(g_event)
            }
          }
        }
        //npackets += 1;
        packet_types.push(header.ptype);
        match search_for_u16(0x90eb, &stream, pos) {
          Err(err) => {
            println!("Unable to find next header! {err}");
            break;
          }
          Ok(head_pos) => {
            pos = head_pos;
          }
        }
      }
    }
  }
  events
}


/// Read serialized TelemetryPackets from an existing file
///
/// Read GAPS binary files ("Berkeley binaries)
#[derive(Debug)]
pub struct TelemetryPacketReader {
  /// Read from this file
  pub filename        : String,
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
    let repr = format!("<TelemetryPacketReader : file {}, read {} packets, filter {}, range {}>", self.filename, self.n_packs_read, self.filter, range_repr);
    write!(f, "{}", repr)
  }
}

impl TelemetryPacketReader {

  pub fn new(filename : String) -> TelemetryPacketReader {
    let fname_c = filename.clone();
    let file = OpenOptions::new().create(false).append(false).read(true).open(fname_c).expect("Unable to open file {filename}");
    let packet_reader = Self { 
      filename,
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

  /// Get an index of the file - count number of packets
  ///
  /// Returns the number of all PacketTypes in the file
  pub fn get_packet_index(&mut self) -> io::Result<HashMap<u8, usize>> {
    let mut index  = HashMap::<u8, usize>::new();
    let mut buffer = [0];
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
      if buffer[0] != 0xeb {
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

        if buffer[0] != 0x90 { 
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
          let ptype    = TelemetryPacketType::from(buffer[0]);
          let mut padding = [0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut padding) {
            Err(err) => {
              error!("Unable to read from file! {err}");
              break;
            }
            Ok(_) => {
              self.cursor += 6;
            }
          }
          // read the the size of the packet

          let mut buffer_psize = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              error!("Unable to read from file! {err}");
              break;
            }
            Ok(_) => {
              self.cursor += 4;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let mut size = parse_u32(&vec_data, &mut 0);
          // This size includes the header
          if (size as usize) < TelemetryHeader::SIZE {
            error!("This packet might be empty or corrupt!");
            break;
          }
          size -= TelemetryHeader::SIZE as u32;

          match self.file_reader.seek(SeekFrom::Current(size as i64)) {
            Err(err) => {
              debug!("Unable to read more data! {err}");
              break; 
            }
            Ok(_) => {
              self.cursor += size as usize;
              // and then we add the packet type to the 
              // hashmap
              let ptype_key = ptype as u8;
              if index.contains_key(&ptype_key) {
                *index.get_mut(&ptype_key).unwrap() += 1;
              } else {
                index.insert(ptype_key, 1usize);
              }
            }
          }
        }
      } // if no 0xAA found
    } // end loop
    self.rewind()?;
    Ok(index)
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
  pub fn get_next_packet(&mut self) -> Option<TelemetryPacket> {
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
      if buffer[0] != 0xeb {
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

        if buffer[0] != 0x90 { 
          continue;
        } else {
          // the 3rd byte is the packet type
          match self.file_reader.read_exact(&mut buffer) {
             Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
            }
            Ok(_) => {
              self.cursor += 1;
            }
          }
          let mut thead = TelemetryHeader::new();
          thead.sync      = 0x90eb;
          thead.ptype     = buffer[0];
          let ptype    = TelemetryPacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_ts = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_ts) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
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
              return None;
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
              return None;
            }
            Ok(_) => {
              self.cursor += 2;
              thead.checksum    = u16::from_le_bytes(buffer_checksum);
            }
          }
          
          let mut size     = thead.length;
          // This size includes the header
          if (size as usize) < TelemetryHeader::SIZE {
            error!("This packet might be empty or corrupt!");
            return None;
          }
          size -= TelemetryHeader::SIZE as u16;
          if ptype != self.filter && self.filter != TelemetryPacketType::Unknown {
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
          

          let mut tp = TelemetryPacket::new();
          tp.header  = thead;
          
          //tp.packet_type = ptype;
          //let mut payload = vec![0u8;TelemetryHeader::SIZE];
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
              return None;
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

impl Iterator for TelemetryPacketReader {
  type Item = TelemetryPacket;
  
  fn next(&mut self) -> Option<Self::Item> {
    self.get_next_packet()
  }
}


