// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;


#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum FitStatus {
  Unknown        = 0u8,
  DidNotConverge = 10u8,
  Success        = 42u8,
}

expand_and_test_enum!(FitStatus, test_fitstatus_repr);


/// Describe line depending on z since that is our
/// best constrained value
///
/// This model has 6 free parameters, 3 for 
/// the anchor point and 3 for the direction
pub fn line3d(z : f32, x_a : f32, y_a : f32, z_a : f32, dx : f32, dy : f32, dz : f32) -> (f32, f32, f32) {
  // avoid zero division error 
  //let dx_nz : f32 = dx if dx !=0 {dx} else { 1e-5 };
  //let dy_nz : f32 = dy if dy !=0 {dy} else { 1e-5 };
  let dz_nz : f32 = if dz != 0.0 {dz} else { 1e-5 };
  let x = x_a + ((dx/dz_nz) * (z - z_a));
  let y = y_a + ((dy/dz_nz) * (z - z_a));
  (x,y,z)
}

pub struct LineFit {
}















