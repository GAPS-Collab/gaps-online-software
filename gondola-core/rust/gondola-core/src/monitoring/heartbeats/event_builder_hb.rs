// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use colored::Colorize;

//use std::fmt;
//#[cfg(feature = "random")]  
//use crate::random::FromRandom;
//#[cfg(feature = "random")]  
//use rand::Rng;
//
//
//use crate::io::serialization::Serialization;
//use crate::io::parsers::{
//  parse_u64,
//};
//
//use crate::monitoring::MoniData;
//
//use crate::errors::SerializationError;
//
//use crate::packets::{
//  TofPackable,
//  TofPacketType,
//};
//
//#[cfg(feature="pybindings")]
//use pyo3::prelude::*;
//
//#[cfg(feature="pybindings")]
//use pyo3::exceptions::{
//  PyIOError,
//  PyKeyError
//};
//
//#[cfg(feature="pybindings")]
//use crate::packets::TofPacket;
//
//use crate::moniseries;
//
//#[cfg(feature="pybindings")]
//use crate::{
//  pythonize_packable,
//  pythonize_monidata
//};



#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct EventBuilderHB {
  /// Mission elapsed time in seconds
  pub met_seconds           : u64,
  /// Total number of received MasterTriggerEvents (from MTB)
  pub n_mte_received_tot    : u64,
  /// Total number of received RBEvents (from all RB)
  pub n_rbe_received_tot    : u64,
  /// Average number of RBEvents per each MTEvent
  pub n_rbe_per_te          : u64,
  /// Total number of discarded RBEvents (accross all boards)
  pub n_rbe_discarded_tot   : u64,
  /// TOtal number of missed MTEvents. "Skipped means" gaps in 
  /// consecutive rising event ids
  pub n_mte_skipped         : u64,
  /// Total number of events that timed out, which means they 
  /// got send before all RBEvents could be associated with 
  /// this event
  pub n_timed_out           : u64,
  /// Total number of events passed on to the gloabl data sink 
  /// thread
  pub n_sent                  : u64,
  /// ?
  pub delta_mte_rbe           : u64,
  /// The total size of the current event cache in number of events
  pub event_cache_size        : u64,
  /// In paralel to the event_cache, the event_id cache holds event ids.
  /// This should be perfectly aligned to the event_cache by design.
  pub event_id_cache_size     : u64, 
  /// The total number of hits which we lost due to the DRS being busy
  /// (this is on the Readoutboards)
  pub drs_bsy_lost_hg_hits    : u64,
  /// The total number of RBEvents which do not have a MasterTriggerEvent
  pub rbe_wo_mte              : u64,
  /// The current length of the channel which we use to send events from 
  /// the MasterTrigger thread to the event builder
  pub mte_receiver_cbc_len    : u64,
  /// The current length of the channel whcih we use for all readoutboard
  /// threads to send their events to the event builder
  pub rbe_receiver_cbc_len    : u64,
  /// the current length of the channel which we use to send built events 
  /// to the global data sink thread
  pub tp_sender_cbc_len       : u64,
  /// The total number of RBEvents which have an event id which is SMALLER
  /// than the smallest event id in the event cache. 
  pub n_rbe_from_past         : u64,
  pub n_rbe_orphan            : u64,
  // let's deprecate this!
  pub n_rbe_per_loop          : u64,
  /// The totabl number of events with the "AnyDataMangling" flag set
  pub data_mangled_ev         : u64,
  // pub seen_rbevents         : HashMap<u8, usize>,
}

impl EventBuilderHB {
  // pub fn new() -> Self {
  //   let mut seen_rbevents = HashMap::<u8, usize>::new();
  //   for k in 1..47 {
  //     if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
  //       continue;
  //     } else {
  //       seen_rbevents.insert(k as u8, 0);
  //     }
  //   }
  pub fn new() -> Self {
    Self {
      met_seconds          : 0,
      n_mte_received_tot   : 0,
      n_rbe_received_tot   : 0,
      n_rbe_per_te         : 0,
      n_rbe_discarded_tot  : 0,
      n_mte_skipped        : 0,
      n_timed_out          : 0,
      n_sent               : 0,
      delta_mte_rbe        : 0,
      event_cache_size     : 0,
      event_id_cache_size  : 0,
      drs_bsy_lost_hg_hits : 0,
      rbe_wo_mte           : 0,
      mte_receiver_cbc_len : 0,
      rbe_receiver_cbc_len : 0,
      tp_sender_cbc_len    : 0,
      n_rbe_per_loop       : 0,
      n_rbe_orphan         : 0,
      n_rbe_from_past      : 0,
      data_mangled_ev      : 0,
      // seen_rbevents        : seen_rbevents, 
    }
  }


  /// The average number of RBEvents per
  /// TofEvent, tis is the average number
  /// of active ReadoutBoards per TofEvent
  pub fn get_average_rbe_te(&self) -> f64 {
   if self.n_sent > 0 {
     return self.n_rbe_per_te as f64 / self.n_sent as f64;
   }
   0.0
  }

  pub fn get_timed_out_frac(&self) -> f64 {
    if self.n_sent > 0 {
      return self.n_timed_out as f64 / self.n_sent as f64;
    }
    0.0
  }

  // pub fn add_rbevent(&mut self, rb_id : u8) {
  //   *self.seen_rbevents.get_mut(&rb_id).unwrap() += 1;
  // }
  
  pub fn get_incoming_vs_outgoing_mte(&self) -> f64 {
    if self.n_sent > 0 {
      return self.n_mte_received_tot as f64 /  self.n_sent as f64;
    }
    0.0
  }

  pub fn get_nrbe_discarded_frac(&self) -> f64 {
    if self.n_rbe_received_tot > 0 {
     return self.n_rbe_discarded_tot as f64 / self.n_rbe_received_tot as f64;
   }
   0.0
  }
  
  pub fn get_mangled_frac(&self) -> f64 {
    if self.n_mte_received_tot > 0 {
     return self.data_mangled_ev as f64 / self.n_mte_received_tot as f64;
   }
   0.0
  }

  pub fn get_drs_lost_frac(&self) -> f64 {
    if self.n_rbe_received_tot > 0 {
      return self.drs_bsy_lost_hg_hits as f64 / self.n_rbe_received_tot as f64;
    }
    0.0
  }

  pub fn pretty_print(&self) -> String {
    let mut repr = String::from("");
    repr += &(format!("\n \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} EVENTBUILDER HEARTBTEAT \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} "));
    repr += &(format!("\n Mission elapsed time (MET) [s]      : {}", self.met_seconds).bright_purple());
    repr += &(format!("\n Num. events sent                    : {}", self.n_sent).bright_purple());
    repr += &(format!("\n Size of event cache                 : {}", self.event_cache_size).bright_purple());
    //repr += &(format!("\n Size of event ID cache              : {}", self.event_id_cache_size).bright_purple());
    repr += &(format!("\n Num. events timed out               : {}", self.n_timed_out).bright_purple());
    repr += &(format!("\n Percent events timed out            : {:.2}%", self.get_timed_out_frac()*(100 as f64)).bright_purple());
    //if self.n_sent > 0 && self.n_rbe_per_loop > 0 {
    //  repr += &(format!("\n Percent events w/out event ID : {:.2}%", (((self.n_rbe_per_loop / self.n_sent) as f64)*(100 as f64))).bright_purple());
    //} else if self.n_rbe_per_loop > 0 { 
    //  repr += &(format!("\n Percent events w/out event ID : N/A").bright_purple());
    //}
    if self.n_mte_received_tot > 0{ 
      repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
      repr += &(format!("\n Num. evts with ANY data mangling  : {}"     , self.data_mangled_ev));
      repr += &(format!("\n Per. evts with ANY data mangling  : {:.2}%" , self.get_mangled_frac()*(100 as f64)));
    }
    else {repr += &(format!("\n Percent events with data mangling: unable to calculate"));}
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Received MTEvents                   : {}", self.n_mte_received_tot).bright_purple());
    repr += &(format!("\n Skipped MTEvents                    : {}", self.n_mte_skipped).bright_purple());
    repr += &(format!("\n Incoming/outgoing MTEvents fraction : {:.2}", self.get_incoming_vs_outgoing_mte()).bright_purple());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Received RBEvents                   : {}", self.n_rbe_received_tot).bright_purple());
    repr += &(format!("\n RBEvents Discarded                  : {}", self.n_rbe_discarded_tot).bright_purple());
    repr += &(format!("\n Percent RBEvents discarded          : {:.2}%", self.get_nrbe_discarded_frac()*(100 as f64)).bright_purple());
    repr += &(format!("\n DRS4 busy lost hits                 : {}", self.drs_bsy_lost_hg_hits).bright_purple());
    repr += &(format!("\n RDS4 busy lost hits fraction        : {:.2}%", self.get_drs_lost_frac()*(100.0 as f64)).bright_purple());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    if self.n_sent > 0 && self.n_mte_received_tot > 0 {
        repr += &(format!("\n RBEvent/Evts sent               : {:.2}", (self.n_rbe_received_tot as f64/ self.n_sent as f64)).bright_purple());
        repr += &(format!("\n RBEvent/MTEvents                : {:.2}", (self.n_rbe_received_tot as f64 / self.n_mte_received_tot as f64)).bright_purple()); }
    repr += &(format!("\n Current RBevents / iteration        : {:.2}", self.n_rbe_per_loop).bright_purple());
    repr += &(format!("\n Num. RBEvents with evid from past   : {}",  self.n_rbe_from_past).bright_purple());
    repr += &(format!("\n Num. orphan RBEvents                : {}",  self.n_rbe_orphan).bright_purple());
    repr += &(format!("\n\n Getting MTE from cache for RBEvent failed {} times :(", self.rbe_wo_mte).bright_blue());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Ch. len MTE Receiver                : {}", self.mte_receiver_cbc_len).bright_purple());
    repr += &(format!("\n Ch. len RBE Reveiver                : {}", self.rbe_receiver_cbc_len).bright_purple());
    repr += &(format!("\n Ch. len TP Sender                   : {}", self.tp_sender_cbc_len).bright_purple());
    repr += &(format!("\n \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} END EVENTBUILDER HEARTBTEAT \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50}"));
    repr
  }
}


impl MoniData for EventBuilderHB {
  fn get_board_id(&self) -> u8 {
    0
  }
  
  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"             => Some(0.0),
      "met_seconds"          => Some(self.met_seconds as f32),
      "n_mte_received_tot"   => Some(self.n_mte_received_tot as f32),
      "n_rbe_received_tot"   => Some(self.n_rbe_received_tot as f32),
      "n_rbe_per_te"         => Some(self.n_rbe_per_te as f32),
      "n_rbe_discarded_tot"  => Some(self.n_rbe_discarded_tot as f32),
      "n_mte_skipped"        => Some(self.n_mte_skipped as f32),
      "n_timed_out"          => Some(self.n_timed_out as f32),
      "n_sent"               => Some(self.n_sent as f32),
      "delta_mte_rbe"        => Some(self.delta_mte_rbe as f32),
      "event_cache_size"     => Some(self.event_cache_size as f32),
      "event_id_cache_size"  => Some(self.event_id_cache_size as f32),
      "drs_bsy_lost_hg_hits" => Some(self.drs_bsy_lost_hg_hits as f32),
      "rbe_wo_mte"           => Some(self.rbe_wo_mte as f32),
      "mte_receiver_cbc_len" => Some(self.mte_receiver_cbc_len as f32),
      "rbe_receiver_cbc_len" => Some(self.rbe_receiver_cbc_len as f32),
      "tp_sender_cbc_len"    => Some(self.tp_sender_cbc_len as f32),
      "n_rbe_per_loop"       => Some(self.n_rbe_per_loop as f32),
      "n_rbe_orphan"         => Some(self.n_rbe_orphan as f32),
      "n_rbe_from_past"      => Some(self.n_rbe_from_past as f32),
      "data_mangled_ev"      => Some(self.data_mangled_ev as f32),
      _                      => None
    }
  }

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str> {
    vec!["board_id", "met_seconds", "n_mte_received_tot",
         "n_rbe_received_tot", "n_rbe_per_te", "n_rbe_discarded_tot",
         "n_mte_skipped", "n_timed_out", "n_sent", "delta_mte_rbe",
         "event_cache_size", "event_id_cache_size","drs_bsy_lost_hg_hits",
         "rbe_wo_mte", "mte_receiver_cbc_len", "rbe_receiver_cbc_len",
         "tp_sender_cbc_len", "n_rbe_per_loop", "n_rbe_orphan", "n_rbe_from_past",
         "data_mangled_ev"]
  }
}

moniseries!(EventBuilderHBSeries,EventBuilderHB);

#[cfg(feature="pybindings")]
#[pymethods]
impl EventBuilderHB {
  /// The average number of RBEvents per
  /// TofEvent, tis is the average number
  /// of active ReadoutBoards per TofEvent
  #[getter]
  #[pyo3(name="average_rbe_te")]
  fn get_average_rbe_te_py(&self) -> f64 {
    self.get_average_rbe_te()
  }


  #[getter]
  #[pyo3(name="timed_out_frac")]
  pub fn get_timed_out_frac_py(&self) -> f64 {
    self.get_timed_out_frac()
  }
 
  #[getter]
  #[pyo3(name="incoming_vs_outgoing_mte")]
  pub fn get_incoming_vs_outgoing_mte_py(&self) -> f64 {
    self.get_incoming_vs_outgoing_mte()
  }

  #[getter]
  #[pyo3(name="nrbe_discarded_frac")]
  pub fn get_nrbe_discarded_frac_py(&self) -> f64 {
    self.get_nrbe_discarded_frac()
  }
  
  #[getter]
  #[pyo3(name="mangled_frac")]
  pub fn get_mangled_frac_py(&self) -> f64 {
    self.get_mangled_frac()
  }

  #[getter]
  #[pyo3(name="drs_lost_frac")]
  pub fn get_drs_lost_frac_py(&self) -> f64 {
    self.get_drs_lost_frac()
  }  
}

#[cfg(feature="pybindings")]
pythonize_monidata!(EventBuilderHB);
#[cfg(feature="pybindings")]
pythonize_packable!(EventBuilderHB);

//-----------------------------------------------------

impl Default for EventBuilderHB {
  fn default () -> Self {
    Self::new()
  }
}

impl TofPackable for EventBuilderHB {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::EventBuilderHB;
}

impl Serialization for EventBuilderHB {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 156; //

  fn from_bytestream(stream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream,pos)?;
    let mut hb = EventBuilderHB::new();
    hb.met_seconds          = parse_u64(stream,pos);
    hb.n_mte_received_tot   = parse_u64(stream,pos);
    hb.n_rbe_received_tot   = parse_u64(stream,pos);
    hb.n_rbe_per_te         = parse_u64(stream,pos);
    hb.n_rbe_discarded_tot  = parse_u64(stream,pos);
    hb.n_mte_skipped        = parse_u64(stream,pos);
    hb.n_timed_out          = parse_u64(stream,pos);
    hb.n_sent               = parse_u64(stream,pos);
    hb.delta_mte_rbe        = parse_u64(stream,pos);
    hb.event_cache_size     = parse_u64(stream,pos);
    //hb.event_id_cache_size  = parse_u64(stream,pos);
    hb.drs_bsy_lost_hg_hits = parse_u64(stream,pos);
    hb.rbe_wo_mte           = parse_u64(stream,pos);
    hb.mte_receiver_cbc_len = parse_u64(stream,pos);
    hb.rbe_receiver_cbc_len = parse_u64(stream,pos);
    hb.tp_sender_cbc_len    = parse_u64(stream,pos);
    hb.n_rbe_per_loop         = parse_u64(stream,pos);
    hb.n_rbe_from_past      = parse_u64(stream,pos);
    hb.n_rbe_orphan         = parse_u64(stream,pos);
    hb.data_mangled_ev      = parse_u64(stream,pos);
    // hb.seen_rbevents        = HashMap::from(parse_u8(stream, pos));
    *pos += 2;
    Ok(hb)
  }
    
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.met_seconds.to_le_bytes());
    bs.extend_from_slice(&self.n_mte_received_tot.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_received_tot.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_per_te.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_discarded_tot.to_le_bytes());
    bs.extend_from_slice(&self.n_mte_skipped.to_le_bytes());
    bs.extend_from_slice(&self.n_timed_out.to_le_bytes());
    bs.extend_from_slice(&self.n_sent.to_le_bytes());
    bs.extend_from_slice(&self.delta_mte_rbe.to_le_bytes());
    bs.extend_from_slice(&self.event_cache_size.to_le_bytes());
    //bs.extend_from_slice(&self.event_id_cache_size.to_le_bytes());
    bs.extend_from_slice(&self.drs_bsy_lost_hg_hits.to_le_bytes());
    bs.extend_from_slice(&self.rbe_wo_mte.to_le_bytes());
    bs.extend_from_slice(&self.mte_receiver_cbc_len.to_le_bytes());
    bs.extend_from_slice(&self.rbe_receiver_cbc_len.to_le_bytes());
    bs.extend_from_slice(&self.tp_sender_cbc_len.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_per_loop.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_from_past.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_orphan.to_le_bytes());
    bs.extend_from_slice(&self.data_mangled_ev.to_le_bytes());
    // bs.push(self.seen_rbevents.to_u8());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature="random")]
impl FromRandom for EventBuilderHB {
  fn from_random() -> Self {
    let mut rng              = rand::rng();
    Self {
      met_seconds            : rng.random::<u64>(),
      n_rbe_received_tot     : rng.random::<u64>(),
      n_rbe_per_te           : rng.random::<u64>(),
      n_rbe_discarded_tot    : rng.random::<u64>(),
      n_mte_skipped          : rng.random::<u64>(),
      n_timed_out            : rng.random::<u64>(),
      n_sent                 : rng.random::<u64>(),
      delta_mte_rbe          : rng.random::<u64>(),
      event_cache_size       : rng.random::<u64>(),
      // don't randomize this, since it 
      // won't get serialized
      event_id_cache_size    :                0,
      drs_bsy_lost_hg_hits   : rng.random::<u64>(),
      rbe_wo_mte             : rng.random::<u64>(),
      mte_receiver_cbc_len   : rng.random::<u64>(),
      rbe_receiver_cbc_len   : rng.random::<u64>(),
      tp_sender_cbc_len      : rng.random::<u64>(),
      n_mte_received_tot     : rng.random::<u64>(),
      n_rbe_per_loop         : rng.random::<u64>(),
      n_rbe_from_past        : rng.random::<u64>(),
      n_rbe_orphan           : rng.random::<u64>(),
      data_mangled_ev        : rng.random::<u64>()
    }
  }
} 

impl fmt::Display for EventBuilderHB {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<EVTBLDRHearbeat:   ");
    repr += &self.pretty_print();
    write!(f, "{}>", repr)
  }
}  

#[cfg(feature="random")]
#[test]
fn pack_eventbuilderhb() {
  for _ in 0..100 {
    let hb = EventBuilderHB::from_random();
    let test : EventBuilderHB = hb.pack().unpack().unwrap();
    assert_eq!(hb, test);
  }
}

