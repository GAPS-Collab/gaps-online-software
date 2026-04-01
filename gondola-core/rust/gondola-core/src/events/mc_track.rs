// The following file is part of gaps-online-software and published 
// under the GPLv3 license


use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McTrack {
  pub hits : Vec<McHit>,
}

impl McTrack {
  pub fn new() -> Self {
    Self {
      hits : Vec::<McHit>::new() 
    }
  }
}
