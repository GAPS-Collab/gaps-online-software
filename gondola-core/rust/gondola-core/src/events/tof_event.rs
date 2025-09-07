//! Basic event structure for all TOF systems
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[cfg(feature="database")]
use std::f32::consts::PI;
use std::cmp::Ordering;

/// Main event class for the TOF. This will be sent over telemetry and be 
/// written to disk
///
/// CHANGELOG: v0.11 (gondola-core): Merges TofEvent, TofEventSummary and 
/// MasterTriggerEvent all into a single TofEvent, based on former TofEventSummary.
/// The new TofEvent has the ability to cary RBEvents and thus mimick the "old" TofEvent
/// We are using the version flag to indicate:
/// * ProtocolVersion::Unknown - The "old" TofEventSummary. (now "TofEvent"). No extra 
///   variables for the GCU, no RBEvents
/// * ProtocolVersion::V1      - The version crafted for Antarctica '24/'25 containing 
///   a bunch of summary variables for the GCU (e.g. nhits(umbrella)). This version will 
///   add these variables to the bytestream and also if ProtocolVersion::V1 is read out 
///   from the bytestream, the variables are expected to be in it. This is to keep compatibility
///   with the gcu
/// * ProtocolVersion::V2     - v0.11 (gondola-core) version of TofEvent(Summary).
///   This version will not write out GCU variables and does not expect them to be in the 
///   bytestream. If desired, this version can read/write RBEvents. 
///
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofEvent {
  pub status            : EventStatus,
  /// The version of this event. Prior to tof-dataclasses v0.11,
  /// the default was `ProtocolVersion::Unknown`. 
  /// Then for the Antarctic campaign we added more variables, 
  /// so that the gcu doesn't have to compute them. These variables
  /// will be included in the bytestream for `ProtocolVersion::V1`.
  /// However, we don't want to write them to disk in the new (gondola-core > v0.11)
  /// verion, so we will use ProtocolVersionV2 for the version to be
  /// written to disk. ProtocolVersionV2 can (but not must) have RBEvents
  pub version           : ProtocolVersion,
  pub quality           : EventQuality,
  pub trigger_sources   : u16,

  /// the number of triggered paddles coming
  /// from the MTB directly. This might NOT be
  /// the same as the number of hits!
  pub n_trigger_paddles  : u8,
  pub event_id           : u32,
  pub run_id             : u16,
  pub timestamp32        : u32,
  pub timestamp16        : u16,
  /// scalar number of hits missed in
  /// this event due to DRS on the RB
  /// being busy
  pub drs_dead_lost_hits : u16, 
  pub dsi_j_mask         : u32,
  pub channel_mask       : Vec<u16>,
  pub mtb_link_mask      : u64,
  pub hits               : Vec<TofHit>,
  // a number of variables which are directly 
  // read out from the MTB packet and won't get 
  // serialized in this form but then used
  // later to calculate other variables
  pub mt_trigger_sources : u16,
  pub mt_tiu_gps16       : u16,
  pub mt_tiu_gps32       : u32, 
  pub mt_timestamp       : u32,
  pub mt_tiu_timestamp   : u32,
  // a bunch of calculated variablels, used 
  // for online interesting event search
  // these will be only available in ProtocolVersion 1
  pub n_hits_umb         : u8,
  pub n_hits_cbe         : u8,
  pub n_hits_cor         : u8,
  pub tot_edep_umb       : f32,
  pub tot_edep_cbe       : f32,
  pub tot_edep_cor       : f32,
  pub paddles_set        : bool,
  // this is new (v0.11) -> We are expanding TofEvent(Summary) 
  // so that it is the same as TofEvent in v0.10 and is able 
  // to carry waveforms. These then can be stripped off
  pub rb_events          : Vec<RBEvent>,
  /// Start time for a time to wait for incoming RBEvents
  pub creation_time     : Instant,
  pub write_to_disk     : bool, 
}

impl TofEvent {

  pub fn new() -> Self {
    Self {
      status             : EventStatus::Unknown,
      version            : ProtocolVersion::Unknown,
      n_hits_umb         : 0,
      n_hits_cbe         : 0,
      n_hits_cor         : 0,
      tot_edep_umb       : 0.0,
      tot_edep_cbe       : 0.0,
      tot_edep_cor       : 0.0,
      quality            : EventQuality::Unknown,
      trigger_sources    : 0,
      n_trigger_paddles  : 0,
      event_id           : 0,
      run_id             : 0,
      timestamp32        : 0,
      timestamp16        : 0,
      drs_dead_lost_hits : 0,
      dsi_j_mask         : 0,
      channel_mask       : Vec::<u16>::new(),
      mtb_link_mask      : 0,
      hits               : Vec::<TofHit>::new(),
      mt_trigger_sources : 0,
      mt_tiu_gps16       : 0,
      mt_tiu_gps32       : 0, 
      mt_timestamp       : 0,
      mt_tiu_timestamp   : 0,
      paddles_set        : false,
      rb_events          : Vec::<RBEvent>::new(),
      creation_time      : Instant::now(),
      write_to_disk      : true,
    }
  }
 
  /// Remove any RBEvents from the event 
  pub fn strip_rbevents(&mut self) {
    self.rb_events.clear();
  }
  
  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }
  
  /// Simple check if the event contains as much RBEvents 
  /// as expected from the provided boards masks by the MTB
  pub fn is_complete(&self) -> bool {
    self.get_rb_link_ids().len() == self.rb_events.len()
  }
  
  /// The number of hits we did not get 
  /// becaue of the DRS busy
  pub fn get_lost_hits(&self) -> u16 {
    let mut lost_hits = 0u16;
    for rbev in &self.rb_events {
      if rbev.header.drs_lost_trigger() {
        let mut nhits = rbev.header.get_nchan() as u16;
        if nhits > 0 {
          nhits -= 1;
        }
        lost_hits += nhits;
      }
    }
    lost_hits
  }


  /// Calculate extra variables for the GCU, 
  /// set the protocol version to V1 and 
  /// strip the waveforms if desired
  /// 
  /// # Arguments:
  ///   * strip_rbevents : remove the rbevents from the TofEvent so they
  ///                      won't bother the poor gcu
  pub fn prepare_for_gcu(&mut self, strip_rbevents : bool) {
    if strip_rbevents {
      self.strip_rbevents();
    }
    self.version = ProtocolVersion::V1;
    for h in &self.hits {
      if h.paddle_id <= 60 {
        self.n_hits_cbe += 1;
        self.tot_edep_cbe += h.get_edep();
      }
      else if h.paddle_id <= 108 && h.paddle_id > 60 {
        self.n_hits_umb += 1;
        self.tot_edep_umb += h.get_edep();
      }
      else {
        self.n_hits_cor += 1;
        self.tot_edep_cor += h.get_edep();
      }
    }
  }
  
  /// Ensure compatibility with older data, which 
  /// contained a different type of TofEvent
  pub fn decode_depr_tofevent_size_header(mask : &u32) 
    -> (usize, usize) {
    let rb_event_len = (mask & 0xFF)        as usize;
    let miss_len     = ((mask & 0xFF00)     >> 8)  as usize;
    (rb_event_len, miss_len)
  }

  /// Set timing offsets to the event's hits
  ///
  /// # Arguments:
  ///   * offsets : a hashmap paddle id -> timing offset
  #[cfg(feature="database")]
  pub fn set_timing_offsets(&mut self, offsets : &HashMap<u8, f32>) {
    for h in self.hits.iter_mut() {
      if offsets.contains_key(&h.paddle_id) {
        h.timing_offset = offsets[&h.paddle_id]; 
      }
    }
  }

  /// Remove hits from the hitseries which can not 
  /// be caused by the same particle, which means 
  /// that for these two specific hits beta with 
  /// respect to the first hit in the event is 
  /// larger than one
  /// That this works, first hits need to be 
  /// "normalized" by calling normalize_hit_times
  ///
  /// # Return:
  ///
  ///   * removed paddle ids, twindows
  pub fn lightspeed_cleaning(&mut self, t_err : f32) -> (Vec<u8>, Vec<f32>) {
    // first sort the hits in time
    if self.hits.len() == 0 {
      return (Vec::<u8>::new(), Vec::<f32>::new());
    }
    let mut twindows = Vec::<f32>::new();
    self.hits.sort_by(|a,b| (a.event_t0).partial_cmp(&b.event_t0).unwrap_or(Ordering::Greater));
    let first_hit = self.hits[0].clone(); // the clone here is a bit unfortunate, 
                                           // this can be done better with walking 
                                           // over the list and updating references
    let mut clean_hits = Vec::<TofHit>::new(); 
    let mut rm_hits    = Vec::<u8>::new();
    // per definition, we can't remove the first hit, ever
    clean_hits.push(first_hit.clone());
    //error!("-----");
    let mut prior_hit = first_hit;
    //println!("TO FIRST {}",first_hit.event_t0);
    for h in self.hits.iter().skip(1) {
      let min_tdiff_cvac = 1e9*1e-3*prior_hit.distance(h)/299792458.0;
      let twindow            = prior_hit.event_t0 + min_tdiff_cvac;

      // FIXME - implement different strategies
      // this is the "default" strategy
      //if h.event_t0 < twindow {
      //  rm_hits.push(h.paddle_id);
      //  twindows.push(twindow);
      //  continue;
      //}
      // this is the "lenient" strategy
      if h.event_t0 + 2.0*t_err < twindow {
        rm_hits.push(h.paddle_id);
        twindows.push(twindow);
        continue;
      }
      // this is the "aggressive" strategy
      //if h.event_t0 - 2.0*t_err < twindow {
      //  rm_hits.push(h.paddle_id);
      //  continue;
      //}
      
      // should we remove negative beta hits?
      //if beta < 0.0 {
      //  rm_hits.push(h.paddle_id);
      //  continue;
      //}
      // update the prior hit only 
      // when it is a good one. If it 
      // is bad we continue to compare
      // to the first hit
      prior_hit = h.clone();
      clean_hits.push(*h);
    }
    self.hits = clean_hits;
    (rm_hits, twindows)
  }


  /// Non causal hits have hit times in ends A and be which 
  /// are not compatible with the speed of light in the paddle, 
  /// that is, the hit gets registered too early. If we look 
  /// at a plot of the reconstructed position, these hits would 
  /// correspond to positions outside of the paddle.
  ///
  /// #Returns:
  ///   A vector of paddle ids with removed hits
  ///
  pub fn remove_non_causal_hits(&mut self) -> Vec<u8> {
    let mut clean_hits = Vec::<TofHit>::new();
    let mut removed_pids = Vec::<u8>::new();
    for h in &self.hits {
      if h.obeys_causality() {
        clean_hits.push(*h);
      } else {
        removed_pids.push(h.paddle_id);
      }
    }
    self.hits = clean_hits;
    removed_pids
  }
  
  #[cfg(feature="database")]
  pub fn normalize_hit_times(&mut self) {
    if self.hits.len() == 0 {
      return;
    }
    // check if hit times have already been normalized
    if self.hits[0].event_t0 == 0.0 {
      return;
    }

    let phase0 = self.hits[0].phase.to_f32();
    for h in &mut self.hits {
      let t0 = h.get_t0_uncorrected() + h.get_cable_delay();
      let mut phase_diff = h.phase.to_f32() - phase0;
      while phase_diff < - PI/2.0 {
        phase_diff += 2.0*PI;
      }
      while phase_diff > PI/2.0 {
        phase_diff -= 2.0*PI;
      }
      let t_shift = 50.0*phase_diff/(2.0*PI);
      h.event_t0 = t0 + t_shift;
    }
    // start the first hit at 0
    self.hits.sort_by(|a,b| (a.event_t0).partial_cmp(&b.event_t0).unwrap());
    let t0_first_hit = self.hits[0].event_t0;
    for h in self.hits.iter_mut() {
      h.event_t0 -= t0_first_hit
    }
  }
 
  #[cfg(feature="database")]
  pub fn set_paddles(&mut self, paddles : &HashMap<u8, TofPaddle>) {
    let mut nerror = 0u8;
    for h in &mut self.hits {
      match paddles.get(&h.paddle_id) {
        None => {
          error!("Got paddle id {} which is not in given map!", h.paddle_id);
          nerror += 1;
          continue;
        }
        Some(pdl) => {
          h.set_paddle(pdl);
        }
      }
    }
    if nerror == 0 {
      self.paddles_set = true;
    }
  }

  /// Get the pointcloud of this event, sorted by time
  /// 
  /// # Returns
  ///   (f32, f32, f32, f32, f32) : (x,y,z,t,edep)
  pub fn get_pointcloud(&self) -> Option<Vec<(f32,f32,f32,f32,f32)>> {
    let mut pc = Vec::<(f32,f32,f32,f32,f32)>::new();
    if !self.paddles_set {
      error!("Before getting the pointcloud, paddle information needs to be set for this event. Call TofEvent;:set_paddle");
      return None;
    }
    for h in &self.hits {
      let result = (h.x, h.y, h.z, h.get_t0(), h.get_edep());
      pc.push(result);
    }
    Some(pc)
  }

  /// Compare the MasterTriggerEvent::trigger_hits with 
  /// the actual hits to determine from which paddles
  /// we should have received HG hits (from waveforms)
  /// but we did not get them
  ///
  /// WARNING: The current implementation of this is 
  /// rather slow and not fit for production use
  /// FIXME - rewrite as a closure
  #[cfg(feature="database")]
  pub fn get_missing_paddles_hg(&self, pid_map :   &DsiJChPidMapping) -> Vec<u8> {
    let mut missing = Vec::<u8>::new();
    for th in self.get_trigger_hits() {
      let pid = pid_map.get(&th.0).unwrap().get(&th.1).unwrap().get(&th.2.0).unwrap().0;
      let mut found = false;
      for h in &self.hits {
        if h.paddle_id == pid {
          found = true;
          break
        }
      }
      if !found {
        missing.push(pid);
      }
    }
    missing
  }
  
  /// Get the triggered paddle ids
  ///
  /// Warning, this might be a bit slow
  #[cfg(feature="database")]
  pub fn get_triggered_paddles(&self, pid_map :   &DsiJChPidMapping) -> Vec<u8> {
    let mut paddles = Vec::<u8>::with_capacity(3);
    for th in &self.get_trigger_hits() {
      let pid = pid_map.get(&th.0).unwrap().get(&th.1).unwrap().get(&th.2.0).unwrap().0;
      paddles.push(pid);
    }
    paddles
  }

  /// Get the RB link IDs according to the mask
  pub fn get_rb_link_ids(&self) -> Vec<u8> {
    let mut links = Vec::<u8>::new();
    for k in 0..64 {
      if (self.mtb_link_mask >> k) as u64 & 0x1 == 1 {
        links.push(k as u8);
      }
    }
    links
  }

  /// Get the combination of triggered DSI/J/CH on 
  /// the MTB which formed the trigger. This does 
  /// not include further hits which fall into the 
  /// integration window. For those, se rb_link_mask
  ///
  /// The returned values follow the TOF convention
  /// to start with 1, so that we can use them to 
  /// look up LTB ids in the db.
  ///
  /// # Returns
  ///
  ///   Vec<(hit)> where hit is (DSI, J, CH) 
  pub fn get_trigger_hits(&self) -> Vec<(u8, u8, (u8, u8), LTBThreshold)> {
    let mut hits = Vec::<(u8,u8,(u8,u8),LTBThreshold)>::with_capacity(5); 
    let physical_channels = [(1u8,  2u8), (3u8,4u8), (5u8, 6u8), (7u8, 8u8),
                             (9u8, 10u8), (11u8,12u8), (13u8, 14u8), (15u8, 16u8)];
    //let n_masks_needed = self.dsi_j_mask.count_ones() / 2 + self.dsi_j_mask.count_ones() % 2;
    let n_masks_needed = self.dsi_j_mask.count_ones();
    if self.channel_mask.len() < n_masks_needed as usize {
      error!("We need {} hit masks, but only have {}! This is bad!", n_masks_needed, self.channel_mask.len());
      return hits;
    }
    let mut n_mask = 0;
    trace!("Expecting {} hit masks", n_masks_needed);
    trace!("ltb channels {:?}", self.dsi_j_mask);
    trace!("hit masks {:?}", self.channel_mask); 
    //println!("We see LTB Channels {:?} with Hit masks {:?} for {} masks requested by us!", self.dsi_j_mask, self.channel_mask, n_masks_needed);
    
    // one k here is for one ltb
    for k in 0..32 {
      if (self.dsi_j_mask >> k) as u32 & 0x1 == 1 {
        let mut dsi = 0u8;
        let mut j   = 0u8;
        if k < 5 {
          dsi = 1;
          j   = k as u8 + 1;
        } else if k < 10 {
          dsi = 2;
          j   = k as u8 - 5 + 1;
        } else if k < 15 {
          dsi = 3;
          j   = k as u8- 10 + 1;
        } else if k < 20 {
          dsi = 4;
          j   = k as u8- 15 + 1;
        } else if k < 25 {
          dsi = 5;
          j   = k as u8 - 20 + 1;
        } 
        //let dsi = (k as f32 / 4.0).floor() as u8 + 1;       
        //let j   = (k % 5) as u8 + 1;
        //println!("n_mask {n_mask}");
        let channels = self.channel_mask[n_mask]; 
        for (i,ch) in LTB_CHANNELS.iter().enumerate() {
          //let chn = *ch as u8 + 1;
          let ph_chn = physical_channels[i];
          //let chn = i as u8 + 1;
          //println!("i,ch {}, {}", i, ch);
          let thresh_bits = ((channels & ch) >> (i*2)) as u8;
          //println!("thresh_bits {}", thresh_bits);
          if thresh_bits > 0 { // hit over threshold
            hits.push((dsi, j, ph_chn, LTBThreshold::from(thresh_bits)));
          }
        }
        n_mask += 1;
      } // next ltb
    }
    hits
  }
  
  /// Get the trigger sources from trigger source byte
  pub fn get_trigger_sources(&self) -> Vec<TriggerType> {
    TriggerType::transcode_trigger_sources(self.trigger_sources)
  }
  
  pub fn get_timestamp48(&self) -> u64 {
    ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  }
  
  /// Ttotal energy depostion in the TOF - Umbrella
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep_umbrella(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      if h.paddle_id < 61 || h.paddle_id > 108 {
        continue;
      }
      tot_edep += h.get_edep();
    }
    tot_edep
  }
  
  /// Ttotal energy depostion in the TOF - Umbrella
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep_cube(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      if h.paddle_id > 60 {
        continue;
      }
      tot_edep += h.get_edep();
    }
    tot_edep
  }
  
  /// Ttotal energy depostion in the Cortina
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep_cortina(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      if h.paddle_id < 109 {
        continue;
      }
      tot_edep += h.get_edep();
    }
    tot_edep
  }
  
  /// Ttotal energy depostion in the complete TOF
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      tot_edep += h.get_edep();
    }
    tot_edep
  }

  pub fn get_nhits_umb(&self) -> usize {
    let mut nhit = 0;
    for h in &self.hits {
      if h.paddle_id > 60 && h.paddle_id < 109 {
        nhit += 1;
      }
    }
    nhit
  }
  
  pub fn get_nhits_cbe(&self) -> usize {
    let mut nhit = 0;
    for h in &self.hits {
      if h.paddle_id < 61  {
        nhit += 1;
      }
    }
    nhit
  }
  
  pub fn get_nhits_cor(&self) -> usize {
    let mut nhit = 0;
    for h in &self.hits {
      if h.paddle_id > 108  {
        nhit += 1;
      }
    }
    nhit
  }

  pub fn get_nhits(&self) -> usize {
    self.hits.len()
  }
  
  /// Check if th eassociated RBEvents have any of their
  /// mangling stati set
  pub fn has_any_mangling(&self) -> bool {
    for rbev in &self.rb_events {
      if rbev.status == EventStatus::CellAndChnSyncErrors 
      || rbev.status == EventStatus::CellSyncErrors 
      || rbev.status == EventStatus::ChnSyncErrors {
        return true;
      }
    }
    false
  }
  
  /// Get all waveforms of all RBEvents in this event
  /// ISSUE - Performance, Memory
  /// FIXME - reimplement this things where this
  ///         returns only a reference
  pub fn get_waveforms(&self) -> Vec<RBWaveform> {
    let mut wfs = Vec::<RBWaveform>::new();
    for ev in &self.rb_events {
      for wf in &ev.get_rbwaveforms() {
        wfs.push(wf.clone());
      }
    }
    wfs
  }
}

impl TofPackable for TofEvent {
  // v0.11 TofPacketType::TofEventSummary -> TofPacketType::TofEvent
  const TOF_PACKET_TYPE        : TofPacketType = TofPacketType::TofEvent;
  const TOF_PACKET_TYPE_ALT    : TofPacketType = TofPacketType::TofEventDeprecated;
}

impl Serialization for TofEvent {
  
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    let status_version = self.status as u8 | self.version.to_u8();
    stream.push(status_version);
    stream.extend_from_slice(&self.trigger_sources.to_le_bytes());
    stream.extend_from_slice(&self.n_trigger_paddles.to_le_bytes());
    stream.extend_from_slice(&self.event_id.to_le_bytes());
    // depending on the version, we send the fc event packet
    if self.version == ProtocolVersion::V1 {
      stream.extend_from_slice(&self.n_hits_umb  .to_le_bytes()); 
      stream.extend_from_slice(&self.n_hits_cbe  .to_le_bytes()); 
      stream.extend_from_slice(&self.n_hits_cor  .to_le_bytes()); 
      stream.extend_from_slice(&self.tot_edep_umb.to_le_bytes()); 
      stream.extend_from_slice(&self.tot_edep_cbe.to_le_bytes()); 
      stream.extend_from_slice(&self.tot_edep_cor.to_le_bytes()); 
    }
    stream.extend_from_slice(&(self.quality as u8).to_le_bytes());
    stream.extend_from_slice(&self.timestamp32.to_le_bytes());
    stream.extend_from_slice(&self.timestamp16.to_le_bytes());
    stream.extend_from_slice(&self.run_id.to_le_bytes());
    stream.extend_from_slice(&self.drs_dead_lost_hits.to_le_bytes());
    stream.extend_from_slice(&self.dsi_j_mask.to_le_bytes());
    let n_channel_masks = self.channel_mask.len();
    stream.push(n_channel_masks as u8);
    for k in 0..n_channel_masks {
      stream.extend_from_slice(&self.channel_mask[k].to_le_bytes());
    }
    stream.extend_from_slice(&self.mtb_link_mask.to_le_bytes());
    let nhits = self.hits.len() as u16;
    stream.extend_from_slice(&nhits.to_le_bytes());
    for k in 0..self.hits.len() {
      stream.extend_from_slice(&self.hits[k].to_bytestream());
    }
    // for the new (>=v0.11) event, we will always write 
    // the rb events
    if self.version == ProtocolVersion::V2 {
      stream.push(self.rb_events.len() as u8);
      for rbev in &self.rb_events {
        stream.extend_from_slice(&rbev.to_bytestream());
      }
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    let mut event           = Self::new();
    let head = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    let status_version_u8     = parse_u8(stream, pos);
    let status                = EventStatus::from(status_version_u8 & 0x3f);
    let version               = ProtocolVersion::from(status_version_u8 & 0xc0); 
    event.status            = status;
    event.version           = version;
    event.trigger_sources   = parse_u16(stream, pos);
    event.n_trigger_paddles = parse_u8(stream, pos);
    event.event_id          = parse_u32(stream, pos);
    if event.version == ProtocolVersion::V1 {
      event.n_hits_umb      = parse_u8(stream, pos); 
      event.n_hits_cbe      = parse_u8(stream, pos); 
      event.n_hits_cor      = parse_u8(stream, pos); 
      event.tot_edep_umb    = parse_f32(stream, pos); 
      event.tot_edep_cbe    = parse_f32(stream, pos); 
      event.tot_edep_cor    = parse_f32(stream, pos); 
    }
    event.quality            = EventQuality::from(parse_u8(stream, pos));
    event.timestamp32        = parse_u32(stream, pos);
    event.timestamp16        = parse_u16(stream, pos);
    event.run_id             = parse_u16(stream, pos);
    event.drs_dead_lost_hits = parse_u16(stream, pos);
    event.dsi_j_mask         = parse_u32(stream, pos);
    let n_channel_masks        = parse_u8(stream, pos);
    for _ in 0..n_channel_masks {
      event.channel_mask.push(parse_u16(stream, pos));
    }
    event.mtb_link_mask     = parse_u64(stream, pos);
    let nhits                 = parse_u16(stream, pos);
    for _ in 0..nhits {
      event.hits.push(TofHit::from_bytestream(stream, pos)?);
    }
    if event.version == ProtocolVersion::V2 {
      // for this version, we can have rb events 
      let n_rb_events = parse_u8(stream, pos);
      if n_rb_events > 0 {
        for _ in 0..n_rb_events {
          event.rb_events.push(RBEvent::from_bytestream(stream, pos)?);
        }
      }
    }

    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(event)
  }
  
  /// Allows to get TofEvent from a packet 
  /// of the deprecate packet type TofEventDeprecated.
  /// This packet type was formerly known as TofEvent
  /// This will dismiss all the 
  /// waveforms and RBEvents
  fn from_bytestream_alt(stream    : &Vec<u8>, 
                         pos       : &mut usize) 
    -> Result<Self, SerializationError> {
    let head    = parse_u16(stream, pos);
    if head != TofEvent::HEAD {
      return Err(SerializationError::HeadInvalid);
    }
    let mut te            = Self::new();
    // the compression level will always be 0 for old data
    let _compression_level = parse_u8(stream, pos);

    te.quality            = EventQuality::from(parse_u8(stream, pos));
    // at this position is the serialized TofEventHeader. We don't have that anymore (>v0.11). 
    // However, the only information we need from it is the run id, the other fields are anyway
    // empty
    *pos += 2; // for TofEventHeader::HEAD
    // FIXME - potentially dangerous for u16 overflow!
    te.run_id             = parse_u32(stream, pos) as u16;
    *pos += 43 - 6;// rest of TofEventHeader 
    //let header         = TofEventHeader::from_bytestream(stream, &mut pos)?;
    // now parse the "old" MasterTriggerEvent
    *pos += 2; // MasterTriggerEvent::HEAD
    let event_status   = parse_u8 (stream, pos);
    te.status              = EventStatus::from(event_status);
    if te.has_any_mangling() {
      te.status = EventStatus::AnyDataMangling;
    }
    te.event_id           = parse_u32(stream, pos);
    let mtb_timestamp      = parse_u32(stream, pos);
    let tiu_timestamp      = parse_u32(stream, pos);
    let tiu_gps32          = parse_u32(stream, pos);
    let _tiu_gps16          = parse_u16(stream,pos);
    let _crc           = parse_u32(stream, pos);
    let mt_timestamp  = (mt_event_get_timestamp_abs48(mtb_timestamp, tiu_gps32, tiu_timestamp ) as f64/1000.0).floor()  as u64; 
    te.timestamp32         = (mt_timestamp  & 0x00000000ffffffff ) as u32;
    te.timestamp16         = ((mt_timestamp & 0x0000ffff00000000 ) >> 32) as u16;
    te.trigger_sources     = parse_u16(stream, pos);
    te.dsi_j_mask          = parse_u32(stream, pos);
    let n_channel_masks   = parse_u8(stream, pos);
    for _ in 0..n_channel_masks {
      te.channel_mask.push(parse_u16(stream, pos));
    }

    te.mtb_link_mask      = parse_u64(stream, pos);
    let mt_event_tail     = parse_u16(stream, pos);
    if mt_event_tail != Self::TAIL {
      // (tail for mt event was the same)
      error!("Parsed TAIL from MT event is incorrect! Got {} instead of {} at pos {}", mt_event_tail, Self::TAIL, pos);
    }
    //let mt_event      = MasterTriggerEvent::from_bytestream(stream, &mut pos)?;
    let v_sizes           = Self::decode_depr_tofevent_size_header(&parse_u32(stream, pos));
    //println!("{:?}", v_sizes);
    for _ in 0..v_sizes.0 {
      // we are getting all waveforms for now, but we can 
      // discard them later
      let next_rb_event = RBEvent::from_bytestream(stream, pos)?;
      //println!("{}", next_rb_event);
      te.rb_events.push(next_rb_event);
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    //println!("{}",te);
    
    // FIXME - this is slow, use Arc<> instead. However, then make 
    // sure to copy them in case we get rid of the rb evvnts 
    // (or does Arc take care of it) 
    for rbev in &te.rb_events {
      for h in &rbev.hits {
        te.hits.push(*h);
      }
    }
    return Ok(te);
  }
}
    
impl Default for TofEvent {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TofEvent (version {})", self.version);
    repr += &(format!("\n  EventID          : {}", self.event_id));
    repr += &(format!("\n  RunID            : {}", self.run_id));
    repr += &(format!("\n  EventStatus      : {}", self.status));
    repr += &(format!("\n  TriggerSources   : {:?}", self.get_trigger_sources()));
    repr += &(format!("\n  NTrigPaddles     : {}", self.n_trigger_paddles));
    repr += &(format!("\n  DRS dead hits    : {}", self.drs_dead_lost_hits));
    repr += &(format!("\n  timestamp32      : {}", self.timestamp32)); 
    repr += &(format!("\n  timestamp16      : {}", self.timestamp16)); 
    repr += &(format!("\n   |-> timestamp48 : {}", self.get_timestamp48())); 
    //repr += &(format!("\n  PrimaryBeta      : {}", self.get_beta())); 
    //repr += &(format!("\n  PrimaryCharge    : {}", self.primary_charge));
    repr += &(format!("\n  ** ** TRIGGER HITS (DSI/J/CH) [{} LTBS] ** **", self.dsi_j_mask.count_ones()));
    for k in self.get_trigger_hits() {
      repr += &(format!("\n  => {}/{}/({},{}) ({}) ", k.0, k.1, k.2.0, k.2.1, k.3));
    }
    repr += "\n  ** ** MTB LINK IDs ** **";
    let mut mtblink_str = String::from("\n  => ");
    for k in self.get_rb_link_ids() {
      mtblink_str += &(format!("{} ", k))
    }
    repr += &mtblink_str;
    repr += &(format!("\n  == Trigger hits {}, expected RBEvents {}",
            self.get_trigger_hits().len(),
            self.get_rb_link_ids().len()));
    repr += &String::from("\n  ** ** ** HITS ** ** **");
    for h in &self.hits {
      repr += &(format!("\n  {}", h));
    }
    if self.rb_events.len() > 0 {
      repr += &format!("\n -- has {} RBEvents with waveforms!", self.rb_events.len());
      repr += "\n -- -- boards: ";
      for b in &self.rb_events {
        repr += &format!("{} ", b.header.rb_id);
      }
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEvent {

  fn from_random() -> Self {
    let mut event           = Self::new();
    let mut rng               = rand::rng();
    let status                = EventStatus::from_random();
    let version               = ProtocolVersion::from_random();
    if version == ProtocolVersion::V1 {
      event.n_hits_umb        = rng.random::<u8>();
      event.n_hits_cbe        = rng.random::<u8>();
      event.n_hits_cor        = rng.random::<u8>();
      event.tot_edep_umb      = rng.random::<f32>();
      event.tot_edep_cbe      = rng.random::<f32>();
      event.tot_edep_cor      = rng.random::<f32>();
      event.quality           = EventQuality::from_random();
    }
    event.status             = status;
    event.version            = version;
    // variable packet for the FC
    event.trigger_sources    = rng.random::<u16>();
    event.n_trigger_paddles  = rng.random::<u8>();
    event.event_id           = rng.random::<u32>();
    event.timestamp32        = rng.random::<u32>();
    event.timestamp16        = rng.random::<u16>();
    event.drs_dead_lost_hits = rng.random::<u16>();
    event.dsi_j_mask         = rng.random::<u32>();
    let n_channel_masks        = rng.random::<u8>();
    for _ in 0..n_channel_masks {
      event.channel_mask.push(rng.random::<u16>());
    }
    event.mtb_link_mask      = rng.random::<u64>();
    //let nhits                  = rng.random::<u8>();
    let nhits: u16 = rng.random_range(0..5);
    for _ in 0..nhits {
      event.hits.push(TofHit::from_random());
    }
    if event.version == ProtocolVersion::V2 {
      let n_rb_events = rng.random_range(0..4);
      for _ in 0..n_rb_events {
        event.rb_events.push(RBEvent::from_random());
      }
    }
    event
  }
}

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TofEvent {
  
  /// Emit a copy of self
  fn copy(&self) -> Self {
    self.clone()
  }

  #[pyo3(name="set_timing_offsets")]
  pub fn set_timing_offsets_py(&mut self, timing_offsets : HashMap<u8, f32>) {
    self.set_timing_offsets(&timing_offsets);
  }
  
  #[pyo3(name="normalize_hit_times")]
  pub fn normalize_hit_times_py(&mut self) {
    self.normalize_hit_times();
  }

  /// Remove hits from the hitseries which can not 
  /// be caused by the same particle, which means 
  /// that for these two specific hits beta with 
  /// respect to the first hit in the event is 
  /// larger than one
  /// That this works, first hits need to be 
  /// "normalized" by calling normalize_hit_times
  #[pyo3(name="lightspeed_cleaning")]
  pub fn lightspeed_cleaning_py(&mut self, t_err : f32) -> (Vec<u16>, Vec<f32>) {
    // return Vec<u16> here so that python does not 
    // interpret it as a byte
    let mut pids = Vec::<u16>::new();
    let (pids_rm, twindows) = self.lightspeed_cleaning(t_err);
    for pid in pids_rm {
      pids.push(pid as u16);
    }
    (pids, twindows)
  }
 

  /// Remove all hits from the event's hit series which 
  /// do NOT obey causality. that is where the timings
  /// measured at ends A and B can not be correlated
  /// by the assumed speed of light in the paddle
  #[pyo3(name="remove_non_causal_hits")]
  fn remove_non_causal_hits_py(&mut self) -> Vec<u16> {
    // return Vec<u16> here so that python does not 
    // interpret it as a byte
    let mut pids = Vec::<u16>::new();
    for pid in self.remove_non_causal_hits() {
      pids.push(pid as u16);
    }
    pids
  }
  
  #[getter]
  fn pointcloud(&self) -> Option<Vec<(f32,f32,f32,f32,f32)>> {
    self.get_pointcloud()
  }

  #[getter]
  #[pyo3(name="has_any_mangling")]
  fn has_any_mangling_py(&self) -> bool {
    self.has_any_mangling() 
  }

  #[getter]
  fn get_event_id(&self) -> u32 {
    self.event_id
  }
  
  #[getter]
  fn get_event_status(&self) -> EventStatus {
    self.status
  }
  
  /// Compare the hg hits of the event with the triggered paddles and 
  /// return the paddles which have at least a missing HG hit
  #[pyo3(name="get_missing_paddles_hg")]
  fn get_missing_paddles_hg_py(&self, mapping : DsiJChPidMapping) -> Vec<u8> {
    self.get_missing_paddles_hg(&mapping)
  }

  /// Get all the paddle ids which have been triggered
  #[pyo3(name="get_triggered_paddles")]
  fn get_triggered_paddles_py(&self, mapping : DsiJChPidMapping) -> Vec<u8> {
    self.get_triggered_paddles(&mapping)
  }

  /// The hits we were not able to read out because the DRS4 chip
  /// on the RBs was busy
  #[getter]
  fn lost_hits(&self) -> u16 {
    self.drs_dead_lost_hits
  }

  /// RB Link IDS (not RB ids) which fall into the 
  /// trigger window
  #[getter]
  fn rb_link_ids(&self) -> Vec<u32> {
    self.get_rb_link_ids().into_iter().map(|byte| byte as u32).collect()
  }

  /// The event might have RBEvents associated with it
  #[getter]
  fn get_rb_events(&self) -> Vec<RBEvent> {
    self.rb_events.clone()
  }

  /// Hits which formed a trigger
  #[getter]
  pub fn trigger_hits(&self) -> PyResult<Vec<(u8, u8, (u8, u8), LTBThreshold)>> {
    Ok(self.get_trigger_hits())
  }
  
  /// The active triggers in this event. This can be more than one, 
  /// if multiple trigger conditions are satisfied.
  #[getter]
  pub fn trigger_sources(&self) -> Vec<TriggerType> {
    self.get_trigger_sources()
  } 

  #[getter]
  #[pyo3(name="hits")]
  pub fn hits_py<'_py>(&self) -> Vec<TofHit> {
  //pub fn hits_py<'_py>(&self) -> PyResult<Bound<'_,Vec<TofHit>>> {
    //Bound::new(py, self.hits)
    //FIXMEFIXMEFIXME
    self.hits.clone()
  }
  
  #[getter]
  #[pyo3(name="hitmap")]
  pub fn hitmap<'_py>(&self) -> HashMap<u8,TofHit> {
  //pub fn hits_py<'_py>(&self) -> PyResult<Bound<'_,Vec<TofHit>>> {
    //Bound::new(py, self.hits)
    //FIXMEFIXMEFIXME
    let mut hitmap = HashMap::<u8, TofHit>::new();
    for h in &self.hits {
      hitmap.insert(h.paddle_id, *h);
    }
    hitmap
  }
  
  /// Total energy depostion in the Umbrella
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  #[getter]
  #[pyo3(name="get_edep_umbrella")]
  pub fn get_edep_umbrella_py(&self) -> f32 {
    self.get_edep_umbrella()
  }
  
  /// Total energy depostion in the Cube
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  #[getter]
  #[pyo3(name="get_edep_cube")]
  pub fn get_edep_cube_py(&self) -> f32 {
    self.get_edep_cube()
  }
  
  /// Total energy depostion in the Cortina
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  #[getter]
  #[pyo3(name="get_edep_cortina")]
  pub fn get_edep_cortina_py(&self) -> f32 {
    self.get_edep_cortina()
  }

  /// Total energy depostion in the complete TOF
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  #[getter]
  #[pyo3(name="get_edep")]
  pub fn get_edep_py(&self) -> f32 {
    self.get_edep()
  }

  #[getter]
  #[pyo3(name="nhits")]
  pub fn nhits_py(&self) -> usize {
    self.get_nhits()
  }

  #[getter]
  #[pyo3(name="nhits_umb")]
  pub fn nhits_umb_py(&self) -> usize {
    self.get_nhits_umb()
  }

  #[getter]
  #[pyo3(name="nhits_cbe")]
  fn get_nhits_cbe_py(&self) -> usize {
    self.get_nhits_cbe()
  }
  
  #[getter]
  #[pyo3(name="nhits_cor")]
  fn get_nhits_cor_py(&self) -> usize {
    self.get_nhits_cor()
  }

  #[getter]
  fn get_timestamp16(&self) -> u16 {
    self.timestamp16
  }
  
  #[getter]
  fn get_timestamp32(&self) -> u32 {
    self.timestamp32
  }
  
  #[getter]
  fn timestamp48(&self) -> u64 {
    self.get_timestamp48()
  }
  
  #[getter]
  fn get_status(&self) -> EventStatus {
    self.status
  }

  #[getter]
  #[pyo3(name="waveforms")]
  fn get_waveforms_py(&self) -> Vec<RBWaveform> {
    self.get_waveforms()
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(TofEvent);

//---------------------------------------------------

#[test]
fn packable_tofeventv0() {
  for _ in 0..500 {
    let mut data = TofEvent::from_random();
    if data.version != ProtocolVersion::Unknown {
      continue;
    }
    let mut test : TofEvent = data.pack().unpack().unwrap();
    //println!("{}", data.hits[0]);
    //println!("{}", test.hits[0]);
    // Manually zero these fields, since comparison with nan will fail and 
    // from_random did not touch these
    //println!("{}", data);
    //println!("{}", test);
    let fix_time = Instant::now();
    test.creation_time = fix_time;
    data.creation_time = fix_time;
    for h in &mut test.hits {
      h.paddle_len       = 0.0; 
      h.coax_cable_time  = 0.0; 
      h.hart_cable_time  = 0.0; 
      h.x                = 0.0; 
      h.y                = 0.0; 
      h.z                = 0.0; 
      h.event_t0         = 0.0;
    }
    assert_eq!(data, test);
  }
}  

#[test]
fn packable_tofeventv1() {
  for _ in 0..500 {
    let mut data = TofEvent::from_random();
    if data.version != ProtocolVersion::V1 {
      continue;
    }
    let mut test : TofEvent = data.pack().unpack().unwrap();
    //println!("{}", data.hits[0]);
    //println!("{}", test.hits[0]);
    // Manually zero these fields, since comparison with nan will fail and 
    // from_random did not touch these
    let fix_time = Instant::now();
    test.creation_time = fix_time;
    data.creation_time = fix_time;
    for h in &mut test.hits {
      h.paddle_len       = 0.0; 
      h.coax_cable_time  = 0.0; 
      h.hart_cable_time  = 0.0; 
      h.x                = 0.0; 
      h.y                = 0.0; 
      h.z                = 0.0; 
      h.event_t0         = 0.0;
    }
    assert_eq!(data, test);
  }
}  

#[test]
fn packable_tofeventv2() {
  for _ in 0..500 {
    let mut data = TofEvent::from_random();
    if data.version != ProtocolVersion::V2 {
      continue;
    }
    let mut test : TofEvent = data.pack().unpack().unwrap();
    //println!("{}", data.hits[0]);
    //println!("{}", test.hits[0]);
    // Manually zero these fields, since comparison with nan will fail and 
    // from_random did not touch these
    let fix_time = Instant::now();
    test.creation_time = fix_time;
    data.creation_time = fix_time;
    for h in &mut test.hits {
      h.paddle_len       = 0.0; 
      h.coax_cable_time  = 0.0; 
      h.hart_cable_time  = 0.0; 
      h.x                = 0.0; 
      h.y                = 0.0; 
      h.z                = 0.0; 
      h.event_t0         = 0.0;
    }
    assert_eq!(data, test);
  }
}  


//#[test]
//#[cfg(feature = "random")]
//fn tofevent_sizes_header() {
//  for _ in 0..100 {
//    let data = TofEvent::from_random();
//    let mask = data.construct_sizes_header();
//    let size = TofEvent::decode_size_header(&mask);
//    assert_eq!(size.0, data.rb_events.len());
//    //assert_eq!(size.1, data.missing_hits.len());
//  }
//}

//#[test]
//#[cfg(feature = "random")]
//fn packable_tofevent() {
//  for _ in 0..5 {
//    let data = TofEvent::from_random();
//    let test : TofEvent = data.pack().unpack().unwrap();
//    assert_eq!(data.header, test.header);
//    assert_eq!(data.compression_level, test.compression_level);
//    assert_eq!(data.quality, test.quality);
//    assert_eq!(data.mt_event, test.mt_event);
//    assert_eq!(data.rb_events.len(), test.rb_events.len());
//    //assert_eq!(data.missing_hits.len(), test.missing_hits.len());
//    //assert_eq!(data.missing_hits, test.missing_hits);
//    assert_eq!(data.rb_events, test.rb_events);
//    //assert_eq!(data, test);
//    //println!("{}", data);
//  }
//}



