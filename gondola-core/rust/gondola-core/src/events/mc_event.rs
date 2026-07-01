// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McEvent {
  pub run_id   : u32,
  pub event_id : u32,
  pub mctree   : McTree, 
  pub primary  : Tracklet
}

impl McEvent {

  pub fn new() -> Self {
    Self {
      run_id   : 0,
      event_id : 0,
      mctree   : McTree::new(),
      primary  : Tracklet::new()
    }
  }


  /// Calculate the difference between the first and the last 
  /// hit time in the event
  pub fn get_event_duration(&self) -> f32 {
    let mut first_hit = f32::MAX;
    let mut last_hit  = 0.0f32; 
    if self.mctree.trackmap.len() == 0 {
      return 0.0;
    }
    for k in self.mctree.trackmap.keys() {
      //println!("glob time {}", h.glob_time);
      for h in &self.mctree.trackmap[k].hits { 
        if h.glob_time > last_hit {
          last_hit = h.glob_time; 
        }
        if h.glob_time < first_hit {
          first_hit = h.glob_time;
        }
      }
    }
    return last_hit - first_hit;
  }
}

impl Serialization for McEvent {
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut event = Self::new();
    let head      = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    event.run_id   = parse_u32(stream, pos);
    event.event_id = parse_u32(stream, pos);
    event.primary  = Tracklet::from_bytestream(stream, pos)?;
    let nhits      = parse_u16(stream, pos);
    let mut hits   = Vec::<McHit>::new();
    for _ in 0..nhits {
      hits.push(McHit::from_bytestream(stream, pos)?);
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      //error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    // assemble and sort the mc tree 
    // this consumes the hits! 
    event.mctree.assemble(&mut hits);
    Ok(event) 
  }
}

impl fmt::Display for McEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<McEvent ");
    repr += &(format!("\n run   id : {}",self.run_id));
    repr += &(format!("\n event id : {}",self.event_id));
    repr += "\n === PRIMARY ===";
    repr += &(format!("\n  {}", self.primary));
    //repr += "\n === HITS ===";
    //for h in &self.hits {
    //  repr += &(format!("\n {}", h));
    //}
    repr += "\n === MCTREE ===";
    repr += &(format!("\n {}", self.mctree));
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Frameable for McEvent {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType = CRFrameObjectType::McTree;
}

#[cfg(feature="pybindings")] 
#[pymethods] 
impl McEvent {

  /// The properties of the injected primary 
  #[getter] 
  #[pyo3(name="primary")]
  fn get_primary_py(&self) -> Tracklet {
    self.primary.clone() 
  }

  #[getter] 
  #[pyo3(name="event_duration")]
  fn get_event_duration_py(&self) -> f32 {
    return self.get_event_duration();
  }

  /// Get McTruth Hits in the format 
  /// (x,y,z,time,edep,volumeid,hardware_id)
  #[getter]
  fn get_mctruth_hits(&self) -> Vec<(f32, f32, f32, f32, f32, u32, u32)> {
    let mut hits = Vec::<(f32,f32,f32,f32,f32,u32,u32)>::new();
    for track_id in self.mctree.trackmap.keys() {
      for h in &self.mctree.trackmap[track_id].hits {
        let hp = (h.pos_x, h.pos_y, h.pos_z, h.glob_time,h.kin_E,h.hw_id, h.volume_id);
        hits.push(hp);
      }
    }
    return hits;
  }
  
  /// Combine all energy depositions in a certain volume 
  #[getter]
  fn get_volid_edep_map(&self) -> HashMap<u32, f32> {
    let mut edep_map = HashMap::<u32,f32>::new();
    for track_id in self.mctree.trackmap.keys() {
      for h in &self.mctree.trackmap[track_id].hits {
        if edep_map.contains_key(&h.volume_id) {
          *edep_map.get_mut(&h.volume_id).unwrap() += h.step_edep; 
        } else {
          edep_map.insert(h.volume_id, h.step_edep);
        }
      }
    }
    return edep_map;
  }
  
  /// Combine all energy depositions in a certain volume 
  #[getter]
  fn get_hwid_edep_map(&self) -> HashMap<u32, f32> {
    let mut edep_map = HashMap::<u32,f32>::new();
    for track_id in self.mctree.trackmap.keys() {
      for h in &self.mctree.trackmap[track_id].hits {
        if edep_map.contains_key(&h.hw_id) {
          *edep_map.get_mut(&h.hw_id).unwrap() += h.step_edep; 
        } else {
          edep_map.insert(h.hw_id, h.step_edep);
        }
      }
    }
    return edep_map;
  }
  
  #[getter]
  #[pyo3(name="mctree")]
  fn get_mctree_py(&self) -> McTree { 
    self.mctree.clone()
  }

}

#[cfg(feature="pybindings")]
pythonize!(McEvent);

