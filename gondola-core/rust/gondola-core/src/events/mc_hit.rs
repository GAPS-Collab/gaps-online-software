// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
#[allow(non_snake_case)]
pub struct McHit {
  pub volume_id    : u32,
  pub hw_id        : u32,
  pub parent_id    : u32,
  pub track_id     : u32,
  pub kin_E        : f32,
  pub glob_time    : f32,
  pub pos_x        : f32,
  pub pos_y        : f32,
  pub pos_z        : f32,
  pub vertex_pos_x : f32,
  pub vertex_pos_y : f32,
  pub vertex_pos_z : f32,
  #[allow(non_snake_case)]
  pub vertex_kin_E : f32,
  pub mom_x        : f32,
  pub mom_y        : f32,
  pub mom_z        : f32,
  pub vertex_mom_x : f32,
  pub vertex_mom_y : f32,
  pub vertex_mom_z : f32,
  pub step_len     : f32,
  pub pre_mom_x    : f32,
  pub pre_mom_y    : f32,
  pub pre_mom_z    : f32,
  #[allow(non_snake_case)]
  pub pre_kin_E    : f32,
  
  pub pdg                   : i32,
  pub pre_step_status       : i32,
  pub post_step_status      : i32,
  pub vertex_vol_id         : u32,
  pub vertex_hw_id          : u32,
  pub is_first_step_in_vol  : bool,
  pub is_last_step_in_vol   : bool,
  pub process_type          : u8,
}

impl McHit {
  pub fn new() -> Self {
    Self {
      volume_id    : 0,
      hw_id        : 0,
      parent_id    : 0,
      track_id     : 0,
      kin_E        : 0.0,
      glob_time    : 0.0,
      pos_x        : 0.0,
      pos_y        : 0.0,
      pos_z        : 0.0,
      vertex_pos_x : 0.0,
      vertex_pos_y : 0.0,
      vertex_pos_z : 0.0,
      vertex_kin_E : 0.0,
      mom_x        : 0.0,
      mom_y        : 0.0,
      mom_z        : 0.0,
      vertex_mom_x : 0.0,
      vertex_mom_y : 0.0,
      vertex_mom_z : 0.0,
      step_len     : 0.0,
      pre_mom_x    : 0.0,
      pre_mom_y    : 0.0,
      pre_mom_z    : 0.0,
      pre_kin_E    : 0.0,
      
      pdg                   : 0,
      pre_step_status       : 0,
      post_step_status      : 0,
      vertex_vol_id         : 0,
      vertex_hw_id          : 0,
      is_first_step_in_vol  : false,
      is_last_step_in_vol   : false,
      process_type          : 0,
    }
  }
}

impl Serialization for McHit {
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize) 
    -> Result<Self, SerializationError> {
    //let size : usize = 24*4 + 2*16;
    //*pos += size;
    let mut hit = Self::new();
    let head = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    hit.volume_id    = parse_u32(stream, pos);
    hit.hw_id        = parse_u32(stream, pos);
    hit.parent_id    = parse_u32(stream, pos);
    hit.track_id     = parse_u32(stream, pos);
    hit.kin_E        = parse_f32(stream, pos);
    hit.glob_time    = parse_f32(stream, pos);
    hit.pos_x        = parse_f32(stream, pos);
    hit.pos_y        = parse_f32(stream, pos);
    hit.pos_z        = parse_f32(stream, pos);
    hit.vertex_pos_x = parse_f32(stream, pos);
    hit.vertex_pos_y = parse_f32(stream, pos);
    hit.vertex_pos_z = parse_f32(stream, pos);
    hit.vertex_kin_E = parse_f32(stream, pos);
    hit.mom_x        = parse_f32(stream, pos);
    hit.mom_y        = parse_f32(stream, pos);
    hit.mom_z        = parse_f32(stream, pos);
    hit.vertex_mom_x = parse_f32(stream, pos);
    hit.vertex_mom_y = parse_f32(stream, pos);
    hit.vertex_mom_z = parse_f32(stream, pos);
    hit.step_len     = parse_f32(stream, pos);
    hit.pre_mom_x    = parse_f32(stream, pos);
    hit.pre_mom_y    = parse_f32(stream, pos);
    hit.pre_mom_z    = parse_f32(stream, pos);
    hit.pre_kin_E    = parse_f32(stream, pos);
    hit.pdg                  = parse_i32(stream, pos);
    hit.pre_step_status      = parse_i32(stream, pos);
    hit.post_step_status     = parse_i32(stream, pos);
    hit.vertex_vol_id        = parse_u32(stream, pos);
    hit.vertex_hw_id         = parse_u32(stream, pos);
    hit.is_first_step_in_vol = parse_bool(stream, pos);
    hit.is_last_step_in_vol  = parse_bool(stream, pos);
    hit.process_type         = parse_u8(stream, pos);


    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      //error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    
    Ok(hit)
  }
}

impl fmt::Display for McHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<McHit ");
    repr += &(format!("\n volume_id    : {}",self.volume_id   ));
    repr += &(format!("\n hw_id        : {}",self.hw_id       ));
    repr += &(format!("\n parent_id    : {}",self.parent_id   ));
    repr += &(format!("\n track_id     : {}",self.track_id    ));
    repr += &(format!("\n kin_E        : {}",self.kin_E       ));
    repr += &(format!("\n glob_time    : {}",self.glob_time   ));
    repr += &(format!("\n pos_x        : {}",self.pos_x       ));
    repr += &(format!("\n pos_y        : {}",self.pos_y       ));
    repr += &(format!("\n pos_z        : {}",self.pos_z       ));
    repr += &(format!("\n vertex_pos_x : {}",self.vertex_pos_x));
    repr += &(format!("\n vertex_pos_y : {}",self.vertex_pos_y));
    repr += &(format!("\n vertex_pos_z : {}",self.vertex_pos_z));
    repr += &(format!("\n vertex_kin_E : {}",self.vertex_kin_E));
    repr += &(format!("\n mom_x        : {}",self.mom_x       ));
    repr += &(format!("\n mom_y        : {}",self.mom_y       ));
    repr += &(format!("\n mom_z        : {}",self.mom_z       ));
    repr += &(format!("\n vertex_mom_x : {}",self.vertex_mom_x));
    repr += &(format!("\n vertex_mom_y : {}",self.vertex_mom_y));
    repr += &(format!("\n vertex_mom_z : {}",self.vertex_mom_z));
    repr += &(format!("\n step_len     : {}",self.step_len    ));
    repr += &(format!("\n pre_mom_x    : {}",self.pre_mom_x   ));
    repr += &(format!("\n pre_mom_y    : {}",self.pre_mom_y   ));
    repr += &(format!("\n pre_mom_z    : {}",self.pre_mom_z   ));
    repr += &(format!("\n pre_kin_E    : {}",self.pre_kin_E   ));
    repr += &(format!("\n pdg                  : {}", self.pdg                 ));
    repr += &(format!("\n pre_step_status      : {}", self.pre_step_status     ));
    repr += &(format!("\n post_step_status     : {}", self.post_step_status    ));
    repr += &(format!("\n vertex_vol_id        : {}", self.vertex_vol_id       ));
    repr += &(format!("\n vertex_hw_id         : {}", self.vertex_hw_id        ));
    repr += &(format!("\n is_first_step_in_vol : {}", self.is_first_step_in_vol));
    repr += &(format!("\n is_last_step_in_vol  : {}", self.is_last_step_in_vol ));
    repr += &(format!("\n process_type         : {}", self.process_type        ));
    repr += ">";
    write!(f, "{}", repr)
  }
}

pythonize!(McHit);
