// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use colored::Colorize;


#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct EventBuilderHB {
  pub version                 : ProtocolVersion,
  /// Mission elapsed time in seconds
  pub met_seconds             : u64,
  /// Total number of received MasterTriggerEvents (from MTB)
  pub n_mte_received_tot      : u32,
  /// Total number of received RBEvents (from all RB)
  pub n_rbe_received_tot      : u32,
  /// Average number of RBEvents per each MTEvent
  pub n_rbe_per_te            : u32,
  /// Total number of discarded RBEvents (accross all boards)
  pub n_rbe_discarded_tot     : u32,
  /// TOtal number of missed MTEvents. "Skipped means" gaps in 
  /// consecutive rising event ids
  pub n_mte_skipped           : u32,
  /// Total number of events that timed out, which means they 
  /// got send before all RBEvents could be associated with 
  /// this event
  pub n_timed_out             : u32,
  /// Total number of events that timed out for the secondary 
  /// trigger, similar as n_timed out
  pub n_timed_out_combo       : u32,
  /// Total number of events passed on to the gloabl data sink 
  /// thread
  pub n_sent                  : u32,
  /// ?
  pub delta_mte_rbe           : u32,
  /// The total size of the current event cache in number of events
  pub event_cache_size        : u32,
  /// In paralel to the event_cache, the event_id cache holds event ids.
  /// This should be perfectly aligned to the event_cache by design.
  pub event_id_cache_size     : u32, 
  /// The total number of hits which we lost due to the DRS being busy
  /// (this is on the Readoutboards)
  pub drs_bsy_lost_hg_hits    : u32,
  /// The total number of RBEvents which do not have a MasterTriggerEvent
  pub rbe_wo_mte              : u32,
  /// The current length of the channel which we use to send events from 
  /// the MasterTrigger thread to the event builder
  pub mte_receiver_cbc_len    : u32,
  /// The current length of the channel whcih we use for all readoutboard
  /// threads to send their events to the event builder
  pub rbe_receiver_cbc_len    : u32,
  /// the current length of the channel which we use to send built events 
  /// to the global data sink thread
  pub tp_sender_cbc_len       : u32,
  /// The total number of RBEvents which have an event id which is SMALLER
  /// than the smallest event id in the event cache. 
  pub n_rbe_from_past         : u32,
  pub n_rbe_orphan            : u32,
  // let's deprecate this!
  pub n_rbe_per_loop          : u32,
  /// The totabl number of events with the "AnyDataMangling" flag set
  pub data_mangled_ev         : u32,
  // pub seen_rbevents         : HashMap<u8, usize>,
  // this will not get serialized - can be filled by 
  // gcu timestamp 
  pub timestamp               : u64,
  // possible new fields for ProtocolVersion::V1
  // reserved_0 -> used for n_timed_out_sec
  //pub reserved_0              : u32,
  pub n_sent_trigger          : u32,
  pub n_sent_combo_trigger    : u32,
  pub reserved_3              : u32,
  pub reserved_4              : u32,
  pub reserved_5              : u32,
  pub reserved_6              : u32,
  pub reserved_7              : u32,
  pub reserved_8              : u32,
  pub reserved_9              : u32,
  pub reserved_10             : u32,
  pub reserved_11             : u32,
  pub reserved_12             : u32,
  pub reserved_13             : u32,
  pub reserved_14             : u32,
  pub reserved_15             : u32,
  pub reserved_16             : u32,
  pub reserved_17             : u32,
  pub reserved_18             : u32,
  pub reserved_19             : u32,
}

impl EventBuilderHB {
  pub fn new() -> Self {
    Self {
      // from now on, set new heartbeats to version 
      // V1. This allows to read the new fields
      version              : ProtocolVersion::V1,
      met_seconds          : 0,
      n_mte_received_tot   : 0,
      n_rbe_received_tot   : 0,
      n_rbe_per_te         : 0,
      n_rbe_discarded_tot  : 0,
      n_mte_skipped        : 0,
      n_timed_out          : 0,
      n_timed_out_combo    : 0,
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
      timestamp            : 0,
      // used for n_timeout_combo
      //reserved_0           : 0,
      n_sent_trigger       : 0,
      n_sent_combo_trigger : 0,
      reserved_3           : 0,
      reserved_4           : 0,
      reserved_5           : 0,
      reserved_6           : 0,
      reserved_7           : 0,
      reserved_8           : 0,
      reserved_9           : 0,
      reserved_10          : 0,
      reserved_11          : 0,
      reserved_12          : 0,
      reserved_13          : 0,
      reserved_14          : 0,
      reserved_15          : 0,
      reserved_16          : 0,
      reserved_17          : 0,
      reserved_18          : 0,
      reserved_19          : 0
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
      return (self.n_timed_out + self.n_timed_out_combo) as f64  / self.n_sent as f64;
    }
    0.0
  }
  
  pub fn get_timed_out_combo_frac(&self) -> f64 {
    if self.n_sent > 0 {
      return self.n_timed_out_combo as f64 / self.n_sent_combo_trigger as f64;
    }
    0.0
  }
  
  pub fn get_timed_out_trigger_frac(&self) -> f64 {
    if self.n_sent_trigger > 0 {
      return self.n_timed_out as f64 / self.n_sent_trigger as f64;
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
    repr += &(format!("\n Mission elapsed time (MET) [s]             : {}", self.met_seconds).bright_purple());
    repr += &(format!("\n Num. TofEvents sent                        : {}", self.n_sent).bright_purple());
    repr += &(format!("\n"));
    repr += &(format!("\n Size of event cache                        : {}", self.event_cache_size).bright_purple());
    //repr += &(format!("\n Size of event ID cache                     : {}", self.event_id_cache_size).bright_purple());
    repr += &(format!("\n"));
    repr += &(format!("\n Num. TofEvents timed out                   : {}", self.n_timed_out).bright_purple());
    repr += &(format!("\n Num. TofEvents timed out (combo)           : {}", self.n_timed_out_combo).bright_purple());
    repr += &(format!("\n Percent events timed out                   : {:.2}%", self.get_timed_out_trigger_frac()*(100 as f64)).bright_purple());
    repr += &(format!("\n Percent events timed out (combo)           : {:.2}%", self.get_timed_out_combo_frac()*(100 as f64)).bright_purple());
    if self.n_mte_received_tot > 0{ 
      repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
      repr += &(format!("\n Num. evts with ANY data mangling           : {}"     , self.data_mangled_ev));
      repr += &(format!("\n Per. evts with ANY data mangling           : {:.2}%" , self.get_mangled_frac()*(100 as f64)));
    }
    else {repr += &(format!("\n Percent events with data mangling: unable to calculate"));}
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Received MTEvents (from MTB)               : {}", self.n_mte_received_tot).bright_purple());
    repr += &(format!("\n Skipped MTEvents (gaps in rising event ids): {}", self.n_mte_skipped).bright_purple());
    repr += &(format!("\n Num. MTBEvevents / Num. TofEvents sent     : {:.2}", self.get_incoming_vs_outgoing_mte()).bright_purple());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Received RBEvents                          : {}", self.n_rbe_received_tot).bright_purple());
    repr += &(format!("\n RBEvents Discarded Total                   : {}", self.n_rbe_discarded_tot).bright_purple());
    repr += &(format!("\n Percent RBEvents discarded                 : {:.2}%", self.get_nrbe_discarded_frac()*(100 as f64)).bright_purple());
    repr += &(format!("\n DRS4 busy lost RBEvents                    : {}", self.drs_bsy_lost_hg_hits).bright_purple());
    repr += &(format!("\n Percent DRS4 busy lost RBEvents            : {:.2}%", self.get_drs_lost_frac()*(100.0 as f64)).bright_purple());
    repr += &(format!("\n"));
    repr += &(format!("\n Num. RBEvents with evid from past          : {}",  self.n_rbe_from_past).bright_purple());
    repr += &(format!("\n Num. RBEvents with evid from future        : {}",  self.n_rbe_orphan).bright_purple());
    repr += &(format!("\n RBEvents which do not find their MTEvent \n
                      in the channel on the first try                : {}", self.rbe_wo_mte).bright_blue());
    if self.n_sent > 0 && self.n_mte_received_tot > 0 {
        repr += &(format!("\n"));
        repr += &(format!("\n num. RBEvents / num. TofEvts sent           : {:.2}", (self.n_rbe_received_tot as f64/ self.n_sent as f64)).bright_purple());
        repr += &(format!("\n num. RBEvents / num. MTEvents received      : {:.2}", (self.n_rbe_received_tot as f64 / self.n_mte_received_tot as f64)).bright_purple()); }
    repr += &(format!("\n n_rbe_per_loop                                  : {:.2}", self.n_rbe_per_loop).bright_purple());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Ch. len MTE Receiver                      : {}", self.mte_receiver_cbc_len).bright_purple());
    repr += &(format!("\n Ch. len RBE Receiver                      : {}", self.rbe_receiver_cbc_len).bright_purple());
    repr += &(format!("\n Ch. len TP Sender                         : {}", self.tp_sender_cbc_len).bright_purple());
    repr += &(format!("\n \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} END EVENTBUILDER HEARTBTEAT \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50}"));
    repr
  }
}


impl MoniData for EventBuilderHB {
  fn get_board_id(&self) -> u8 {
    0
  }
 
  fn get_timestamp(&self) -> u64 { 
    if self.timestamp == 0 {
      return self.met_seconds;
    } else {  
      return self.timestamp; 
    }
  }

  fn set_timestamp(&mut self, ts : u64) {
    self.timestamp = ts;
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
      "n_timed_out_combo"    => Some(self.n_timed_out_combo as f32),
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
      "timestamp"            => Some(self.timestamp as f32),
      _                      => None
    }
  }

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str> {
    vec!["board_id", "met_seconds", "n_mte_received_tot",
         "n_rbe_received_tot", "n_rbe_per_te", "n_rbe_discarded_tot",
         "n_mte_skipped", "n_timed_out", "n_timed_out_combo", "n_sent", "delta_mte_rbe",
         "n_sent_trigger", "n_sent_combo_trigger",
         "event_cache_size", "event_id_cache_size","drs_bsy_lost_hg_hits",
         "rbe_wo_mte", "mte_receiver_cbc_len", "rbe_receiver_cbc_len",
         "tp_sender_cbc_len", "n_rbe_per_loop", "n_rbe_orphan", "n_rbe_from_past",
         "data_mangled_ev", "timestamp"]
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
  #[pyo3(name="timed_out_combo_frac")]
  pub fn get_timed_out_combo_frac_py(&self) -> f64 {
    self.get_timed_out_combo_frac()
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
    let version_seconds     = parse_u64(stream,pos);
    hb.version              = ProtocolVersion::from(((version_seconds & 0xC000000000000000) >> 56) as u8); 
    let met_seconds         = version_seconds & 0x3FFFFFFFFFFFFFFF;
    hb.met_seconds          = met_seconds;
    //hb.met_seconds          = parse_u64(stream,pos);
    if hb.version == ProtocolVersion::V1 {
      hb.n_mte_received_tot   = parse_u32(stream,pos);
      hb.n_rbe_received_tot   = parse_u32(stream,pos);
      hb.n_rbe_per_te         = parse_u32(stream,pos);
      hb.n_rbe_discarded_tot  = parse_u32(stream,pos);
      hb.n_mte_skipped        = parse_u32(stream,pos);
      hb.n_timed_out          = parse_u32(stream,pos);
      hb.n_sent               = parse_u32(stream,pos);
      hb.delta_mte_rbe        = parse_u32(stream,pos);
      hb.event_cache_size     = parse_u32(stream,pos);
      //hb.event_id_cache_size  = parse_u64(stream,pos);
      hb.drs_bsy_lost_hg_hits = parse_u32(stream,pos);
      hb.rbe_wo_mte           = parse_u32(stream,pos);
      hb.mte_receiver_cbc_len = parse_u32(stream,pos);
      hb.rbe_receiver_cbc_len = parse_u32(stream,pos);
      hb.tp_sender_cbc_len    = parse_u32(stream,pos);
      hb.n_rbe_per_loop       = parse_u32(stream,pos);
      hb.n_rbe_from_past      = parse_u32(stream,pos);
      hb.n_rbe_orphan         = parse_u32(stream,pos);
      hb.data_mangled_ev      = parse_u32(stream,pos);
      hb.n_timed_out_combo    = parse_u32(stream, pos);
      hb.n_sent_trigger       = parse_u32(stream, pos);
      hb.n_sent_combo_trigger = parse_u32(stream, pos);
      hb.reserved_3           = parse_u32(stream, pos);
      hb.reserved_4           = parse_u32(stream, pos);
      hb.reserved_5           = parse_u32(stream, pos);
      hb.reserved_6           = parse_u32(stream, pos);
      hb.reserved_7           = parse_u32(stream, pos);
      hb.reserved_8           = parse_u32(stream, pos);
      hb.reserved_9           = parse_u32(stream, pos);
      hb.reserved_10          = parse_u32(stream, pos);
      hb.reserved_11          = parse_u32(stream, pos);
      hb.reserved_12          = parse_u32(stream, pos);
      hb.reserved_13          = parse_u32(stream, pos);
      hb.reserved_14          = parse_u32(stream, pos);
      hb.reserved_15          = parse_u32(stream, pos);
      hb.reserved_16          = parse_u32(stream, pos);
      hb.reserved_17          = parse_u32(stream, pos);
      //hb.reserved_18          = parse_u32(stream, pos);
      //hb.reserved_19          = parse_u32(stream, pos);
    } else {
      hb.n_mte_received_tot   = parse_u64(stream,pos) as u32;
      hb.n_rbe_received_tot   = parse_u64(stream,pos) as u32;
      hb.n_rbe_per_te         = parse_u64(stream,pos) as u32;
      hb.n_rbe_discarded_tot  = parse_u64(stream,pos) as u32;
      hb.n_mte_skipped        = parse_u64(stream,pos) as u32;
      hb.n_timed_out          = parse_u64(stream,pos) as u32;
      hb.n_sent               = parse_u64(stream,pos) as u32;
      hb.delta_mte_rbe        = parse_u64(stream,pos) as u32;
      hb.event_cache_size     = parse_u64(stream,pos) as u32;
      //hb.event_id_cache_size  = parse_u64(stream,po as u32s);
      hb.drs_bsy_lost_hg_hits = parse_u64(stream,pos) as u32;
      hb.rbe_wo_mte           = parse_u64(stream,pos) as u32;
      hb.mte_receiver_cbc_len = parse_u64(stream,pos) as u32;
      hb.rbe_receiver_cbc_len = parse_u64(stream,pos) as u32;
      hb.tp_sender_cbc_len    = parse_u64(stream,pos) as u32;
      hb.n_rbe_per_loop       = parse_u64(stream,pos) as u32;
      hb.n_rbe_from_past      = parse_u64(stream,pos) as u32;
      hb.n_rbe_orphan         = parse_u64(stream,pos) as u32;
      hb.data_mangled_ev      = parse_u64(stream,pos) as u32;
    }
    // hb.seen_rbevents        = HashMap::from(parse_u8(stream, pos));
    *pos += 2;
    Ok(hb)
  }
    
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    let mut version_seconds = (self.version as u64) << 56; 
    version_seconds = version_seconds | self.met_seconds;
    //panic!("{}", ((version_seconds & 0xC000000000000000) >> 62) as u8);
    bs.extend_from_slice(&version_seconds.to_le_bytes());
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
    //if self.version == ProtocolVersion::V1 {
    bs.extend_from_slice(&self.n_timed_out_combo.to_le_bytes());
    bs.extend_from_slice(&self.n_sent_trigger.to_le_bytes());
    bs.extend_from_slice(&self.n_sent_combo_trigger.to_le_bytes());
    bs.extend_from_slice(&self.reserved_3.to_le_bytes());
    bs.extend_from_slice(&self.reserved_4.to_le_bytes());
    bs.extend_from_slice(&self.reserved_5.to_le_bytes());
    bs.extend_from_slice(&self.reserved_6.to_le_bytes());
    bs.extend_from_slice(&self.reserved_7.to_le_bytes());
    bs.extend_from_slice(&self.reserved_8.to_le_bytes());
    bs.extend_from_slice(&self.reserved_9.to_le_bytes());
    bs.extend_from_slice(&self.reserved_10.to_le_bytes());
    bs.extend_from_slice(&self.reserved_11.to_le_bytes());
    bs.extend_from_slice(&self.reserved_12.to_le_bytes());
    bs.extend_from_slice(&self.reserved_13.to_le_bytes());
    bs.extend_from_slice(&self.reserved_14.to_le_bytes());
    bs.extend_from_slice(&self.reserved_15.to_le_bytes());
    bs.extend_from_slice(&self.reserved_16.to_le_bytes());
    bs.extend_from_slice(&self.reserved_17.to_le_bytes());
    //bs.extend_from_slice(&self.reserved_18.to_le_bytes());
    //bs.extend_from_slice(&self.reserved_19.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature="random")]
impl FromRandom for EventBuilderHB {
  fn from_random() -> Self {
    let mut rng              = rand::rng();
    // this test can only do ProtocolVersion V1
    let version = ProtocolVersion::V1;
    Self {
      // FIXME 
      version                : version,
      met_seconds            : rng.random::<u64>() & 0x3FFFFFFFFFFFFFFF,
      n_rbe_received_tot     : rng.random::<u32>(),
      n_rbe_per_te           : rng.random::<u32>(),
      n_rbe_discarded_tot    : rng.random::<u32>(),
      n_mte_skipped          : rng.random::<u32>(),
      n_timed_out            : rng.random::<u32>(),
      n_timed_out_combo      : rng.random::<u32>(),
      n_sent                 : rng.random::<u32>(),
      delta_mte_rbe          : rng.random::<u32>(),
      event_cache_size       : rng.random::<u32>(),
      // don't randomize this, since it 
      // won't get serialized
      event_id_cache_size    :                   0,

      drs_bsy_lost_hg_hits   : rng.random::<u32>(),
      rbe_wo_mte             : rng.random::<u32>(),
      mte_receiver_cbc_len   : rng.random::<u32>(),
      rbe_receiver_cbc_len   : rng.random::<u32>(),
      tp_sender_cbc_len      : rng.random::<u32>(),
      n_mte_received_tot     : rng.random::<u32>(),
      n_rbe_per_loop         : rng.random::<u32>(),
      n_rbe_from_past        : rng.random::<u32>(),
      n_rbe_orphan           : rng.random::<u32>(),
      data_mangled_ev        : rng.random::<u32>(),
      timestamp              : 0, // this will get set later or 
                                  // used by MoniSeries
      
      //reserved_0             : 0,
      //reserved_1             : 0,
      //reserved_2             : 0,
      //reserved_3             : 0,
      //reserved_4             : 0,
      //reserved_5             : 0,
      //reserved_6             : 0,
      //reserved_7             : 0,
      //reserved_8             : 0,
      //reserved_9             : 0,
      //reserved_10            : 0,
      //reserved_11            : 0,
      //reserved_12            : 0,
      //reserved_13            : 0,
      //reserved_14            : 0,
      //reserved_15            : 0,
      //reserved_16            : 0,
      //reserved_17            : 0,
      reserved_18            : 0,
      reserved_19            : 0,
      //reserved_0             : rng.random::<u32>(),
      n_sent_trigger         : rng.random::<u32>(),
      n_sent_combo_trigger   : rng.random::<u32>(),
      reserved_3             : rng.random::<u32>(),
      reserved_4             : rng.random::<u32>(),
      reserved_5             : rng.random::<u32>(),
      reserved_6             : rng.random::<u32>(),
      reserved_7             : rng.random::<u32>(),
      reserved_8             : rng.random::<u32>(),
      reserved_9             : rng.random::<u32>(),
      reserved_10            : rng.random::<u32>(),
      reserved_11            : rng.random::<u32>(),
      reserved_12            : rng.random::<u32>(),
      reserved_13            : rng.random::<u32>(),
      reserved_14            : rng.random::<u32>(),
      reserved_15            : rng.random::<u32>(),
      reserved_16            : rng.random::<u32>(),
      reserved_17            : rng.random::<u32>(),
      //reserved_18            : rng.random::<u32>(),
      //reserved_19            : rng.random::<u32>(),
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

