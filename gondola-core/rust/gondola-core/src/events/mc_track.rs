// The following file is part of gaps-online-software and published 
// under the GPLv3 license


use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McTrack {
  pub hits  : Vec<McHit>,
  pub track : Tracklet
}

impl McTrack {
  pub fn new() -> Self {
    Self {
      hits  : Vec::<McHit>::new(),
      track : Tracklet::new()
    }
  }
}

impl fmt::Display for McTrack {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<McTrcak");
    repr += &(format!("\n  {}", self.track));
    for h in &self.hits { 
      repr += &(format!("\n  --   {}", h));
    }
    write!(f,"{}>", repr)
  } 
}

#[cfg(feature="pybindings")]
pythonize!(McTrack);

#[cfg(feature="pybindings")]
#[pymethods]
impl McTrack {
  
  #[getter] 
  fn get_hits(&self) -> Vec<McHit> {
    self.hits.clone()
  }
}



