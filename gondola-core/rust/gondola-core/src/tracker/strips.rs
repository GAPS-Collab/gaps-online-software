// This file is part of gaps-online-software and published 
// under the GPLv3 license

#[cfg(feature="pybindings")]
pub use pyo3::prelude::*; 

use crate::constants::SILI_RADIUS;

/// A helper to plot detector strip
///
/// Assuming there ia a single SiLi-waver at 0,0, return positions and 
/// line lengths to indicate the grooves which separate the strips with 
/// lines. These can then be used by either matplotlib.pyplothlines or
/// matplotlib.pyplot.vlines depending 
/// on the orientation of the wafer
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn strip_lines() -> [(f32, f32);8] {
  let mut strip_lines = [(0.0,0.0);8];   
  let sw  =  [0.2*16.34907456f32/2.0, 0.2* 10.32502464f32/2.0,
              0.2* 9.23299776f32/2.0, 0.2*  8.84362752f32/2.0,
              0.2*-8.84362752f32/2.0, 0.2* -9.23299776f32/2.0,
             -0.2*10.32502464f32/2.0, 0.2*-16.34907456f32/2.0];
  //sw  = 0.2*np.array(sw)
  let mut l_pos = [0.0f32;7];
  l_pos[0] = sw[1] + sw[2] + sw[3];
  l_pos[1] = sw[2] + sw[3];
  l_pos[2] = sw[3];
  l_pos[3] = 0.0;
  l_pos[4] = -1.0*l_pos[2];
  l_pos[5] = -1.0*l_pos[1];
  l_pos[6] = -1.0*l_pos[0];
  let radii = [SILI_RADIUS*0.8,
               SILI_RADIUS*0.95, SILI_RADIUS,
               SILI_RADIUS, SILI_RADIUS,
               SILI_RADIUS*0.95,
               SILI_RADIUS*0.8];
  for k in 0..l_pos.len() {
    strip_lines[k] = (radii[k], l_pos[k]);
  }
  strip_lines
}


