// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McEvent {
  pub run_id   : u32,
  pub event_id : u32,
  pub hits     : Vec<McHit>,
  pub mctree   : McTree, 
  pub primary  : Tracklet
}

impl McEvent {

  pub fn new() -> Self {
    Self {
      run_id   : 0,
      event_id : 0,
      hits     : Vec::<McHit>::new(),
      mctree   : McTree::new(),
      primary  : Tracklet::new()
    }
  }

  fn create_mc_tree(&mut self) {
    self.mctree.assemble(&mut self.hits);
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
    let nhits      = parse_u16(stream, pos);
    for _ in 0..nhits {
      event.hits.push(McHit::from_bytestream(stream, pos)?);
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      //error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    // assemble and sort the mc tree 
    //event.mctree.assemble(event.hits);
    event.create_mc_tree();
    // fill primary properties from the tree
    if event.mctree.trackmap.contains_key(&0) {
      if event.mctree.trackmap[&0].len() > 0 {
        //self.primary.is_infinite   

        event.primary.vertex_mom_x  = event.mctree.trackmap[&0][0].vertex_mom_x;
        event.primary.vertex_mom_y  = event.mctree.trackmap[&0][0].vertex_mom_y;
        event.primary.vertex_mom_z  = event.mctree.trackmap[&0][0].vertex_mom_z;
        event.primary.vertex_x      = event.mctree.trackmap[&0][0].vertex_pos_x;
        event.primary.vertex_y      = event.mctree.trackmap[&0][0].vertex_pos_y;
        event.primary.vertex_z      = event.mctree.trackmap[&0][0].vertex_pos_z;
        event.primary.vertex_energy = event.mctree.trackmap[&0][0].vertex_kin_E;
      }
    }
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
    repr += "\n === HITS ===";
    for h in &self.hits {
      repr += &(format!("\n {}", h));
    }
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

  /// Get McTruth Hits in the format 
  /// (x,y,z,time,edep,volumeid,hardware_id)
  #[getter]
  fn get_mctruth_hits(&self) -> Vec<(f32, f32, f32, f32, f32, u32, u32)> {
    let mut hits = Vec::<(f32,f32,f32,f32,f32,u32,u32)>::new();
    for track_id in self.mctree.trackmap.keys() {
      for h in &self.mctree.trackmap[track_id] {
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
      for h in &self.mctree.trackmap[track_id] {
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
      for h in &self.mctree.trackmap[track_id] {
        if edep_map.contains_key(&h.hw_id) {
          *edep_map.get_mut(&h.hw_id).unwrap() += h.step_edep; 
        } else {
          edep_map.insert(h.hw_id, h.step_edep);
        }
      }
    }
    return edep_map;
  }
}

#[cfg(feature="pybindings")]
pythonize!(McEvent);

