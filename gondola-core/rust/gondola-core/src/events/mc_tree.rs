// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McTree {
  pub tracks : Vec<McTrack>,
}

impl McTree {
  pub fn new() -> Self {
    Self {
      tracks : Vec::<McTrack>::new()
    }
  }

  // create all the tracks and create the actual 
  // tree
  pub fn assemble(&mut self, mut hits : Vec<McHit>) {
    let mut trackmap = HashMap::<u32, Vec<McHit>>::new();
    let mut nhits = hits.len();
    while nhits > 0 {
      let h = hits.pop().unwrap();
      if trackmap.contains_key(&h.track_id) {
        trackmap.get_mut(&h.track_id).unwrap().push(h);
      } else {
        trackmap.insert(h.track_id, vec![h]);
      }
      nhits -= 1;
    }
  }
}



