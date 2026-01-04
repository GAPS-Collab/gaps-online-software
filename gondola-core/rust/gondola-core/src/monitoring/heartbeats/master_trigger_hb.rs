// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use colored::Colorize;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct MasterTriggerHB {
  pub version             : ProtocolVersion, 
  pub total_elapsed       : u64, //aka met (mission elapsed time)
  pub trigger_type        : TriggerType,
  pub combo_trig_type     : TriggerType,
  pub n_events            : u64,
  pub evq_num_events_last : u64,
  pub evq_num_events_avg  : u64,
  pub n_ev_unsent         : u64,
  pub n_ev_missed         : u64,
  pub trate               : u64,
  pub lost_trate          : u64,
  pub clock_rate          : u64,
  pub rb_lost_rate        : u64,
  // these will be available for ProtocolVersion::V1
  pub prescale_track      : f32,
  pub prescale_gaps       : f32,
  pub tiu_ignore_deadtime : bool,
  pub tiu_timeout_cnt     : u64,
  pub tiu_busy_rate       : u16,
  pub trg_lost_trg_rate   : u16,
  pub gaps_blocked_rate   : u16,
  pub track_blocked_rate  : u16,
  pub any_blocked_rate    : u16,
  pub trkctrl_blocked_rate: u16,
  pub trkumbctrl_blocked  : u16,
  pub prescale_bypass     : bool 
}

impl MasterTriggerHB {
  pub fn new() -> Self {
    Self {
      version             : ProtocolVersion::Unknown,
      total_elapsed       : 0,
      trigger_type        : TriggerType::Unknown,
      combo_trig_type     : TriggerType::Unknown,
      n_events            : 0,
      evq_num_events_last : 0,
      evq_num_events_avg  : 0,
      n_ev_unsent         : 0,
      n_ev_missed         : 0,
      trate               : 0,
      lost_trate          : 0,
      clock_rate          : 0,
      rb_lost_rate        : 0,
      // available for protocol version V1 and larger
      prescale_track      : 0.0,
      prescale_gaps       : 0.0,
      tiu_ignore_deadtime : false,
      tiu_timeout_cnt     : 0,
      tiu_busy_rate       : 0,
      trg_lost_trg_rate   : 0,
      gaps_blocked_rate   : 0,
      track_blocked_rate  : 0,
      any_blocked_rate    : 0,
      trkctrl_blocked_rate: 0,
      trkumbctrl_blocked  : 0,
      prescale_bypass     : false
    }
  }

  pub fn get_sent_packet_rate(&self) -> f64 {
    if self.total_elapsed > 0 {
      return self.n_events as f64 / self.total_elapsed as f64;
    }
    0.0
  }

  // get the prescale for the secondary trigger
  pub fn get_prescale_track(&self) -> f64 {
    if self.version == ProtocolVersion::Unknown {
      error!("Prescale not available for protocol version < V1!");
      return 0.0;
    }
    return self.prescale_track as f64
  }
  
  // get the prescale for the secondary trigger
  pub fn get_prescale_gaps(&self) -> f64 {
    if self.version == ProtocolVersion::Unknown {
      error!("Prescale not available for protocol version < V1!");
      return 0.0;
    }
    return self.prescale_gaps as f64
  }
  

  pub fn pretty_print(&self) -> String {
    let mut repr = format!("<MasterTriggerHBs (version : {})", self.version);
    repr += &(format!("\n \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} MTB HEARTBEAT \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} "));
    repr += &(format!("\n MET (Mission Elapsed Time)        : {:.1} sec", self.total_elapsed));
    repr += &(format!("\n Primary trigger type              : {}", self.trigger_type));
    repr += &(format!("\n Secondary trigger type            : {}", self.combo_trig_type));  
    if self.version != ProtocolVersion::Unknown {
        repr += &(format!("\n Primary trigger prescale          : {:.4}", self.prescale_gaps));
        repr += &(format!("\n Secondary trigger prescale        : {:.4}", self.prescale_track));
        repr += &(format!("\n Bypass presecale?                 : {}", self.prescale_bypass));
    }
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}")); 
    repr += &(format!("\n total trigger rate, recorded:     : {:.2} Hz", self.n_events as f64 / self.total_elapsed as f64));
    repr += &(format!("\n total trigger rate, from register : {:.2} Hz", self.trate));
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n lost total trg rate, from register: {:.2} Hz", self.lost_trate));
    repr += &(format!("\n trigger lost rate, from register  : {:.2} Hz", self.trg_lost_trg_rate));
    repr += &(format!("\n RB lost rate, from register       : {:.2} Hz", self.rb_lost_rate));
    repr += &(format!("\n TIU lost rate, from register      : {:.2} HZ", self.tiu_busy_rate));
    repr += &(format!("\n"));
    match self.trigger_type {
        TriggerType::Gaps => {
            repr += &(format!("\n GAPS trigger blocked rate     : {}", self.gaps_blocked_rate));
        }
        TriggerType::Track => {
            repr += &(format!("\n Track trigger blocked rate    : {}", self.track_blocked_rate));
        }
        TriggerType::Any => {
            repr += &(format!("\n Any trigger blocked rate      : {}", self.any_blocked_rate));
        }
        TriggerType::TrackCentral => {
            repr += &(format!("\n Track central blocked rate    : {}", self.trkctrl_blocked_rate));
        }
        TriggerType::TrackUmbCentral => {
            repr += &(format!("\n TrackUmbCentral blocked rate  : {}", self.trkumbctrl_blocked));
        }
        _ => {
            repr += &(format!("\n register not implemented for {} trigger blocked rate", self.trigger_type));
        }
    }

    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Num. recorded Events        : {}", self.n_events));
    repr += &(format!("\n MTB clock rate              : {}", self.clock_rate));

    repr += &(format!("\n"));
    repr += &(format!("\n Using TOF fixed deadtime?         : {}", self.tiu_ignore_deadtime));
    repr += &(format!("\n Fixed deadtime (*10ns)      : {}", self.tiu_timeout_cnt));
    
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Last MTB EVQ size           : {}", self.evq_num_events_last));
    repr += &(format!("\n Avg. MTB EVQ size (per 30s ): {:.2}", self.evq_num_events_avg));
    if self.n_ev_unsent > 0 {
        repr += &(format!("\n Num. sent errors        : {}", self.n_ev_unsent).bold());
    }
    if self.n_ev_missed > 0 {
        repr += &(format!("\n Num. missed events      : {}", self.n_ev_missed).bold());
    }
    repr += &(format!("\n \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} END HEARTBEAT \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} "));
    repr
  }
}
  
impl Default for MasterTriggerHB {
  fn default () -> Self {
    Self::new()
  }
}

impl TofPackable for MasterTriggerHB {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::MasterTriggerHB;
}

impl Serialization for MasterTriggerHB {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 111;

  fn from_bytestream(stream    :&Vec<u8>,
                     pos       :&mut usize)
  -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;
    let mut hb = MasterTriggerHB::new(); 
    hb.version                = ProtocolVersion::from(parse_u8(stream, pos) as u8);
    hb.total_elapsed          = parse_u64(stream, pos);
    hb.trigger_type           = TriggerType::from(parse_u8(stream, pos) as u8);
    hb.combo_trig_type        = TriggerType::from(parse_u8(stream, pos) as u8);
    hb.n_events               = parse_u64(stream, pos);
    hb.evq_num_events_last    = parse_u64(stream, pos);
    hb.evq_num_events_avg     = parse_u64(stream, pos);
    hb.n_ev_unsent            = parse_u64(stream, pos);
    hb.n_ev_missed            = parse_u64(stream, pos);
    hb.trate                  = parse_u64(stream, pos);
    hb.lost_trate             = parse_u64(stream, pos);
    hb.clock_rate             = parse_u64(stream, pos);
    hb.rb_lost_rate           = parse_u64(stream, pos);
    hb.prescale_track         = parse_f32(stream, pos);
    hb.prescale_gaps          = parse_f32(stream, pos);
    /*if hb.version == ProtocolVersion::Unknown {
      hb.prescale_gaps  = 0.0;
      hb.prescale_track = 0.0
    }
    */
    hb.tiu_ignore_deadtime    = parse_bool(stream, pos);
    hb.tiu_timeout_cnt        = parse_u64(stream, pos);
    hb.tiu_busy_rate          = parse_u16(stream, pos);
    hb.trg_lost_trg_rate      = parse_u16(stream, pos);
    match hb.trigger_type {
        TriggerType::Gaps => {
            hb.gaps_blocked_rate    = parse_u16(stream, pos);
        }
        TriggerType::Track => {
            hb.track_blocked_rate   = parse_u16(stream, pos);
        }
        TriggerType::Any => {
            hb.any_blocked_rate     = parse_u16(stream, pos);
        }
        TriggerType::TrackCentral => {
            hb.trkctrl_blocked_rate = parse_u16(stream, pos);
        }
        TriggerType::TrackUmbCentral => {
            hb.trkumbctrl_blocked   = parse_u16(stream, pos);
        }
        _ => {
            *pos += 2;
        }
    }
    hb.prescale_bypass        = parse_bool(stream, pos);
    *pos += 2;
    Ok(hb)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.push(self.version as u8);
    bs.extend_from_slice(&self.total_elapsed.to_le_bytes());
    bs.extend_from_slice(&(self.trigger_type as u8).to_le_bytes());
    bs.extend_from_slice(&(self.combo_trig_type as u8).to_le_bytes());
    bs.extend_from_slice(&self.n_events.to_le_bytes());
    bs.extend_from_slice(&self.evq_num_events_last.to_le_bytes());
    bs.extend_from_slice(&self.evq_num_events_avg.to_le_bytes());
    bs.extend_from_slice(&self.n_ev_unsent.to_le_bytes());
    bs.extend_from_slice(&self.n_ev_missed.to_le_bytes());
    bs.extend_from_slice(&self.trate.to_le_bytes());
    bs.extend_from_slice(&self.lost_trate.to_le_bytes());
    bs.extend_from_slice(&self.clock_rate.to_le_bytes());
    bs.extend_from_slice(&self.rb_lost_rate.to_le_bytes());
    bs.extend_from_slice(&self.prescale_track.to_le_bytes());
    bs.extend_from_slice(&self.prescale_gaps.to_le_bytes());
    bs.extend_from_slice(&(self.tiu_ignore_deadtime as u8).to_le_bytes());
    bs.extend_from_slice(&self.tiu_timeout_cnt.to_le_bytes());
    bs.extend_from_slice(&self.tiu_busy_rate.to_le_bytes());
    bs.extend_from_slice(&self.trg_lost_trg_rate.to_le_bytes());
    match self.trigger_type {
        TriggerType::Gaps => {
            bs.extend_from_slice(&self.gaps_blocked_rate.to_le_bytes());
        }
        TriggerType::Track => {
            bs.extend_from_slice(&self.track_blocked_rate.to_le_bytes());
        }
        TriggerType::Any => {
            bs.extend_from_slice(&self.any_blocked_rate.to_le_bytes());
        }
        TriggerType::TrackCentral => {
            bs.extend_from_slice(&self.trkctrl_blocked_rate.to_le_bytes());
        }
        TriggerType::TrackUmbCentral => {
            bs.extend_from_slice(&self.trkumbctrl_blocked.to_le_bytes());
        }
        _ => {
            bs.extend_from_slice(&[0u8; 2]);
        }
    }
    bs.extend_from_slice(&(self.prescale_bypass as u8).to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for MasterTriggerHB {
  fn from_random() -> Self {
    let mut hb = Self::new();
    let mut rng            = rand::rng(); 
    hb.version             = ProtocolVersion::from_random();  
    hb.total_elapsed       = rng.random::<u64>();
    hb.trigger_type        = TriggerType::from_random();
    hb.combo_trig_type     = TriggerType::from_random();
    hb.n_events            = rng.random::<u64>();
    hb.evq_num_events_last = rng.random::<u64>();
    hb.evq_num_events_avg  = rng.random::<u64>();
    hb.n_ev_unsent         = rng.random::<u64>();
    hb.n_ev_missed         = rng.random::<u64>();
    hb.trate               = rng.random::<u16>() as u64;
    hb.lost_trate          = rng.random::<u16>() as u64;
    hb.clock_rate          = rng.random::<u64>();
    hb.rb_lost_rate        = rng.random::<u16>() as u64;
    hb.prescale_track       = rng.random::<f32>();
    hb.prescale_gaps      = rng.random::<f32>();
    hb.tiu_ignore_deadtime = rng.random::<bool>();
    hb.tiu_timeout_cnt     = rng.random::<u64>();
    hb.tiu_busy_rate       = rng.random::<u16>();
    hb.trg_lost_trg_rate   = rng.random::<u16>();
    match hb.trigger_type {
        TriggerType::Gaps => {
            hb.gaps_blocked_rate = rng.random::<u16>();
        }
        TriggerType::Track => {
            hb.track_blocked_rate = rng.random::<u16>();
        }
        TriggerType::Any => {
            hb.any_blocked_rate = rng.random::<u16>();
        }
        TriggerType::TrackCentral => {
            hb.trkctrl_blocked_rate = rng.random::<u16>();
        }
        TriggerType::TrackUmbCentral => {
            hb.trkumbctrl_blocked = rng.random::<u16>();
        }
        _ => {}
    }
    hb.prescale_bypass   = rng.random::<bool>();
    hb
  }
}

impl fmt::Display for MasterTriggerHB {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.pretty_print();
    write!(f, "{}", repr)
  }
} 

impl MoniData for MasterTriggerHB {
  fn get_board_id(&self) -> u8 {
    0
  }
 
  fn get_timestamp(&self) -> u64 {
    self.total_elapsed
  }
  /*
  fn get_timestamp(&self) -> u64 {
    self.timestamp 
  }

  fn set_timestamp(&mut self, ts : u64) { 
    self.timestamp = ts;
  }
  */

  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "total_elapsed"       => Some(self.total_elapsed as f32),
      "n_events"            => Some(self.n_events as f32),
      "evq_num_events_last" => Some(self.evq_num_events_last as f32),
      "evq_num_events_avg"  => Some(self.evq_num_events_avg as f32),
      "n_ev_unsent"         => Some(self.n_ev_unsent as f32),
      "n_ev_missed"         => Some(self.n_ev_missed as f32),
      "trate"               => Some(self.trate as f32), 
      "lost_trate"          => Some(self.lost_trate as f32),
      "prescale_track"      => Some(self.prescale_track as f32),
      "prescale_gaps"       => Some(self.prescale_gaps as f32),
      "clock_rate"          => Some(self.clock_rate as f32),
      "rb_lost_rate"        => Some(self.rb_lost_rate as f32),
      "tiu_timeout_cnt"     => Some(self.tiu_timeout_cnt as f32),
      "tiu_busy_rate"       => Some(self.tiu_busy_rate as f32),
      "trg_lost_trg_rate"   => Some(self.trg_lost_trg_rate as f32),
      "prescale_bypass"     => Some(self.prescale_bypass as u8 as f32),
      "tiu_ignore_deadtime" => Some(self.tiu_ignore_deadtime as u8 as f32),
      "trigger_type"        => Some(self.trigger_type.to_u8() as f32),
      "combo_trig_type"     => Some(self.combo_trig_type.to_u8() as f32), 
      "gaps_blocked_rate"   => Some(self.gaps_blocked_rate as f32), 
      "track_blocked_rate"  => Some(self.track_blocked_rate as f32),
      "any_blocked_rate"    => Some(self.any_blocked_rate as f32), 
      "trkctrl_blocked_rate"=> Some(self.trkctrl_blocked_rate as f32),
      "trkumbctrl_blocked"  => Some(self.trkumbctrl_blocked as f32),
      //"timestamp"           => Some(self.timestamp as f32),
      _                     => None
    }
  }

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str> {
    vec!["total_elapsed", "trigger_type", "combo_trig_type", "n_events",
         "evq_num_events_last", "evq_num_events_avg", "n_ev_unsent",
         "n_ev_missed", "trate", "lost_trate","clock_rate", "rb_lost_rate", "prescale_track",
         "prescale_gaps", "tiu_ignore_deadtime", "tiu_timeout_cnt", "tiu_busy_rate", "trg_lost_trg_rate", 
         "gaps_blocked_rate", "track_blocked_rate", "any_blocked_rate", "trkctrl_blocked_rate",
         "trkumbctrl_blocked", "prescale_bypass"]
  }
}

moniseries!(MasterTriggerHBSeries, MasterTriggerHB);

//-----------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl MasterTriggerHB {

  //    version             

  #[getter]
  fn get_clock_rate(&self) -> u64 {
      self.clock_rate
  }
  #[getter]
  fn get_trigger_type(&self) -> TriggerType {
      self.trigger_type
  }

  #[getter]
  fn get_combo_trig_type(&self) -> TriggerType {
      self.combo_trig_type
  }
  #[getter]
  fn get_rb_lost_rate(&self) -> u64 {
      self.rb_lost_rate
  }

  #[getter]
  fn get_tiu_ignore_deadtime(&self) -> bool {
      self.tiu_ignore_deadtime
  }
  #[getter]
  fn get_tiu_timeout_cnt(&self) -> u64 {
      self.tiu_timeout_cnt
  }
  #[getter]
  fn get_tiu_busy_rate(&self) -> u16 {
      self.tiu_busy_rate
  }
  #[getter]
  fn get_trg_lost_trg_rate(&self) -> u16 {
      self.trg_lost_trg_rate
  }
  #[getter]
  fn get_gaps_blocked_rate(&self) -> u16 {
      self.gaps_blocked_rate
  }

  #[getter]
  fn get_track_blocked_rate(&self) -> u16 {
      self.track_blocked_rate
  }

  #[getter]
  fn get_any_blocked_rate(&self) -> u16 {
      self.any_blocked_rate
  }
  #[getter]
  fn get_track_central_blocked_rate(&self) -> u16 {
      self.trkctrl_blocked_rate
  }
  #[getter]
  fn get_track_umb_central_blocked_rate(&self) -> u16 {
      self.trkumbctrl_blocked
  }
  #[getter]
  fn get_prescale_bypass(&self) -> bool {
      self.prescale_bypass
  }
  #[getter]
  fn get_total_elapsed(&self) -> u64 {
    self.total_elapsed
  }

  #[getter]
  fn get_evq_mum_events_last(&self) -> u64 {
    self.evq_num_events_last
  }

  #[getter]
  fn get_evq_num_events_avg(&self) -> u64 {
    self.evq_num_events_avg
  }
  
  #[getter]
  fn get_n_ev_unsent(&self) -> u64 {
    self.n_ev_unsent
  }

  #[getter]
  fn get_n_ev_missed(&self) -> u64 {
    self.n_ev_missed
  }
  
  #[getter]
  fn get_trate(&self) -> u64 {
    self.trate
  }

  #[getter]
  fn get_lost_trate(&self) -> u64 {
    self.lost_trate
  }

  #[getter]
  #[pyo3(name="get_prescale_track")]
  fn get_prescale_track_py(&self) -> f32 {
    self.prescale_track
  }

  #[getter]
  #[pyo3(name="get_prescale_gaps")]
  fn get_prescale_gaps_py(&self) -> f32 {
    self.prescale_gaps
  }

  /*
  #[getter]
  #[pyo3(name="timestamp")]
  fn get_timestamp_py(&self) -> u64 {
    self.timestamp
  }
  */
}

#[cfg(feature="pybindings")]
pythonize_packable!(MasterTriggerHB);
#[cfg(feature="pybindings")]
pythonize_monidata!(MasterTriggerHB);

//-----------------------------------------------------

#[cfg(feature="random")]
#[test]
fn pack_master_trigger_hb() {
  for _ in 0..100 {
    let hb = MasterTriggerHB::from_random();
    let test : MasterTriggerHB = hb.pack().unpack().unwrap();
    assert_eq!(hb, test);
  }
} 


