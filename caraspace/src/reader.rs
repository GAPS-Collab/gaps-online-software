//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Specifically, this file is part of the i/o system, more specifically
//! the caraspace library. Caraspace provides a system to read files 
//! for the GAPS experiment comprising different sources, specifically 
//! files from the TOF system written to disk as well as telemetry file
//!
//! While written for the GAPS experiment, the caraspace library is 
//! designed in a form that it should be easily adaptable for other 
//! purposes.
//!
//! This file contains the source for CRReader, a device to read a number
//! of "caraspace" files from a given source.

use std::fmt;

use std::fs::{
  self,
  File,
  OpenOptions
};

use std::path::Path;
use std::io::{
  self,
  BufReader,
  Seek,
  SeekFrom,
  Read,
  ErrorKind
};
use regex::Regex;

use indicatif::{
  ProgressBar,
  ProgressStyle
};

use crate::frame::CRFrame;
use crate::serialization::CRSerializeable;
use crate::parsers::*;

/// Read binaries written through the caraspace system
///
/// The file needs to contain subsequent CRFrames.
#[derive(Debug)] // deliberatly don't have a default() method, reader should fail in that case
pub struct CRReader {
  /// Read from this file
  pub filenames        : Vec<String>,
  /// The position of the current worked on file 
  /// in the filenames vector
  pub file_index       : usize,
  /// A simple BufReader for reading generic binary
  /// files
  file_reader          : BufReader<File>,
  /// Current (byte) position in the current file
  /// This gets reset when we switch to a new file
  cursor               : usize,
  /// Number of read packets
  n_packs_read         : usize,
  /// Number of skipped packets
  n_packs_skipped      : usize,
  /// Number of deserialization errors occured
  /// since the beginning of the file
  pub n_errors         : usize,
  /// Skip the first n packets
  pub skip_ahead       : usize,
  /// Stop reading after n packets
  pub stop_after       : usize,
  ///// A container for TOF paddle to associate
  ///// hits with coordinates
  //pub paddles         : HashMap<u8,Paddle>,
  ///// did paddle loading work
  //pub db_loaded       : bool,
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
    let mut repr = String::from("<CRReader :");
    repr += "\n -- files:";
    for k in &self.filenames {
      repr += &format!("\n     -- {k}");
    }
    if self.filenames.len() > 0 {
      repr += &format!("\n  current : {}", self.get_current_filename().unwrap());
    }
    repr += &String::from("\n -- -- -- -- -- -- -- -- -- -- -- --");
    repr += &format!("\n  read {} packets, {} errors, range {}>", self.n_packs_read, self.n_errors, range_repr);
    write!(f, "{}", repr)
  }
}

impl CRReader {
 
  /// Create a new CRReader
  ///
  /// # Arguments:
  ///   * filename_or_directory : Can be either the name of a single file, or a directory with 
  ///                             caraspace files in it.
  ///   
  pub fn new(filename_or_directory : String) -> Result<Self, io::Error> {
    //let mut paddles   = HashMap::<u8, Paddle>::new();
    //let db_path       = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());
    //let mut db_loaded = false;
    //match connect_to_db(db_path) {
    //  Err(_err) => {
    //    error!("Database can not be found! Did you load the setup-env.sh shell?");
    //  }
    //  Ok(mut conn) => {
    //    match Paddle::all(&mut conn) {
    //      None => {
    //        error!("Unable to retrieve paddle information from DB!");
    //      }
    //      Some(pdls) => {
    //        db_loaded = true;
    //        for p in pdls {
    //          paddles.insert(p.paddle_id as u8, p.clone());
    //        }
    //      }
    //    }
    //  }
    //}
    // check the input argument and get the filelist
    let infiles   = Self::list_path_contents_sorted(&filename_or_directory, None)?;
    if infiles.len() == 0 {
      error!("Unable to read files from {filename_or_directory}. Is this a valid path?");
      return Err(io::Error::new(ErrorKind::NotFound, "Unable to find given path!"))
    }
    let firstfile = infiles[0].clone(); 
    let file = OpenOptions::new().create(false).append(false).read(true).open(&firstfile).expect("Unable to open file {filename}");
    let packet_reader = Self { 
      filenames        : infiles,
      file_index       : 0,
      file_reader      : BufReader::new(file),
      cursor           : 0,
      n_packs_read     : 0,
      n_errors         : 0,
      skip_ahead       : 0,
      stop_after       : 0,
      n_packs_skipped  : 0,
      //paddles         : paddles,
      //db_loaded       : db_loaded
    };
    Ok(packet_reader)
  } 
  
  /// This is the file the current cursor is located 
  /// in and frames are currently read out from 
  pub fn get_current_filename(&self) -> Option<String> {
    // should only happen when it is empty
    if self.filenames.len() <= self.file_index {
      return None;
    }
    Some(self.filenames[self.file_index].clone())
  }
  
  /// Get all filenames in the current path sorted by timestamp if available
  /// If the given path is a file and not a directory, return only that 
  /// file instead
  ///
  /// # Arguments:
  ///
  ///    * input   : name of the target directory
  ///    * patterh : the regex pattern to look for. That the sorting works,
  ///                the pattern needs to return a date for the first
  ///                captured argument and a time for the second captured argument
  fn list_path_contents_sorted(input: &str, pattern: Option<Regex>) -> Result<Vec<String>, io::Error> {
    let path = Path::new(input);
    match fs::metadata(path) {
      Ok(metadata) => {
        if metadata.is_file() {
          let fname = String::from(input);
          return Ok(vec![fname]);
        } 
        if metadata.is_dir() {
          let re : Regex;
          match pattern {
            None => {
              re = Regex::new(r"Run\d+_\d+\.(\d{6})_(\d{6})UTC(\.tof)?\.gaps$").unwrap();
            }
            Some(_re) => {
              re = _re;
            }
          }
          let mut entries: Vec<(u32, u32, String)> = fs::read_dir(path)?
            .filter_map(Result::ok) // Ignore unreadable entries
            .filter_map(|entry| {
              let filename = format!("{}/{}", path.display(), entry.file_name().into_string().ok()?);
              re.captures(&filename.clone()).map(|caps| {
                let date = caps.get(1)?.as_str().parse::<u32>().ok()?;
                let time = caps.get(2)?.as_str().parse::<u32>().ok()?;
                Some((date, time, filename))
              })?
            })
            .collect();

          // Sort by (date, time)
          entries.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
          // Return only filenames
          return Ok(entries.into_iter().map(|(_, _, name)| name).collect());
        } 
        Err(io::Error::new(ErrorKind::Other, "Path exists but is neither a file nor a directory"))
      }
      Err(e) => Err(e),
    }
  }
  
  ///// Use the associated database to enrich paddle information
  //fn add_paddleinfo(&self, event : &mut TofEventSummary) {
  //  event.set_paddles(&self.paddles);
  //}
  
  /// Get the very first frame in all avaialbe files
  pub fn first_frame(&mut self) -> Option<CRFrame> {
    match self.rewind() {
      Err(err) => {
        error!("Error when rewinding files! {err}");
      }
      Ok(_) => ()
    }
    let frame = self.get_next_frame();
    match self.rewind() {
      Err(err) => {
        error!("Error when rewinding files! {err}");
      }
      Ok(_) => ()
    }
    return frame;
  }

  /// Get the very last frame of all infiles
  pub fn last_frame(&mut self) -> Option<CRFrame> { 
    self.file_index = self.filenames.len() - 1;
    let lastfilename = self.filenames[self.file_index].clone();
    let lastfile     = OpenOptions::new().create(false).append(false).read(true).open(lastfilename).expect("Unable to open file {nextfilename}");
    self.file_reader = BufReader::new(lastfile);
    self.cursor      = 0;
    let mut frame = CRFrame::new();
    let mut idx = 0;
    loop {
      match self.get_next_frame() {
        None => {
          match self.rewind() {
            Err(err) => {
              error!("Error when rewinding files! {err}");
            }
            Ok(_) => ()
          }
          if idx == 0 {
            return None;
          } else {
            return Some(frame);
          }
        }
        Some(_fr) => {
          idx += 1;
          frame = _fr;
          continue;
        }
      }
    }
  }

  /// Preview the number of frames in this reader
  pub fn get_n_frames(&mut self) -> usize {
    let _ = self.rewind();
    let mut nframes = 0usize;
    let mut buffer  = [0];
    let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
    let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
    let bar = ProgressBar::new(self.filenames.len() as u64);
    bar.set_position(0);
    bar.set_message (String::from("Counting frames.."));
    bar.set_prefix  ("\u{2728}");
    bar.set_style   (bar_style);
    bar.set_position(self.file_index as u64);
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          match self.progress_file() {
            None    => break,
            Some(_) => {
              bar.set_position(self.file_index as u64);
              continue;
            }
          };
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
            match self.progress_file() {
              None    => break,
              Some(_) => {
                bar.set_position(self.file_index as u64);
                continue;
              }
            };
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }
        // check if the second byte of the header
        if buffer[0] != 0xAA { 
          continue;
        } else {
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              match self.progress_file() {
                None    => break,
                Some(_) => {
                  bar.set_position(self.file_index as u64);
                  continue;
                }
              }
            }
            Ok(_) => {
              self.cursor += 8;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u64(&vec_data, &mut 0);
          match self.file_reader.seek(SeekFrom::Current(size as i64)) {
            Err(err) => {
              error!("Unable to read {size} bytes from {}! {err}", self.get_current_filename().unwrap());
              match self.progress_file() {
                None    => break,
                Some(_) => {
                  bar.set_position(self.file_index as u64);
                  continue;
                }
              }
            }
            Ok(_) => {
              self.cursor += size as usize;
              nframes += 1;
            }
          }
        }
      } // if no 0xAA found
    } // end loop
    bar.finish_with_message("Done!");
    let _ = self.rewind();
    nframes
  } // end fn

  /// Move on to the next file, in case the current one 
  /// is exhausted
  /// 
  /// Return true if there are still files lef
  fn progress_file(&mut self) -> Option<()> {
    if self.file_index == self.filenames.len() -1 {
      return None;
    } else {
      self.file_index += 1;
      let nextfilename = self.filenames[self.file_index].clone();
      let nextfile     = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
      self.file_reader = BufReader::new(nextfile);
      self.cursor      = 0;
      return Some(());
    }
  }


  /// Reset the current state of the reader and make 
  /// the next frame return to be the first frame
  pub fn rewind(&mut self) -> io::Result<()> {
    let firstfile = &self.filenames[0];
    let file      = OpenOptions::new().create(false).append(false).read(true).open(&firstfile)?; 
    self.file_reader  = BufReader::new(file);
    self.file_index = 0;
    self.cursor     = 0;
    Ok(())
  }

  /// Return the next frame for the current files
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  pub fn get_next_frame(&mut self) -> Option<CRFrame> {
    // filter::Unknown corresponds to allowing any

    let mut buffer = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          // this is ok in case we are out of files
          self.progress_file()?;
          return self.get_next_frame();
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
            self.progress_file()?;
            return self.get_next_frame();
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }

        if buffer[0] != 0xAA { 
          continue;
        } else {
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.progress_file()?;
              return self.get_next_frame();
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
                self.progress_file()?;
                return self.get_next_frame();
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
                self.progress_file()?;
                return self.get_next_frame();
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
              self.progress_file()?;
              return self.get_next_frame();
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
              self.progress_file()?;
              return self.get_next_frame();
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

impl Iterator for CRReader {
  type Item = CRFrame;

  fn next(&mut self) -> Option<Self::Item> {
    self.get_next_frame()
  }
}

