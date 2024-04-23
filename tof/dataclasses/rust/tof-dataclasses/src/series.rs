//! Compact data structures for analysis. Won't get serialized. 
//! Summarize data of all boards and provide a more consice
//! table-like structure for analysis.
//! Currently under a lot of development. The goal is to get
//! something which easily translates to a polars data frame.

use std::collections::{
    HashMap,
    VecDeque,
};

use crate::monitoring::{
    HasBoardId,
    RBMoniData,
    LTBMoniData,
    PAMoniData,
    PBMoniData
};

pub trait Series<T> 
  where T : Copy + HasBoardId {

  fn get_data(&self) -> &HashMap<u8,VecDeque<T>>;

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<T>>;
 
  fn get_max_size(&self) -> usize;

  /// Add another instance of the data container to the series
  fn add(&mut self, data : T) {
    let board_id = data.get_board_id();
    if !self.get_data().contains_key(&board_id) {
      self.get_data_mut().insert(board_id, VecDeque::<T>::new());
    } 
    self.get_data_mut().get_mut(&board_id).unwrap().push_back(data);
    if self.get_data_mut().get_mut(&board_id).unwrap().len() > self.get_max_size() {
      error!("The queue is too large, returning the first element! If you need a larger series size, set the max_size field");
      self.get_data_mut().get_mut(&board_id).unwrap().pop_front();
    }
  }
  
  fn get_last_moni(&self, board_id : u8) -> Option<T> {
    let size = self.get_data().get(&board_id)?.len();
    Some(self.get_data().get(&board_id).unwrap()[size - 1])
  }

}

#[derive(Debug, Clone, PartialEq)]
pub struct PAMoniDataSeries {
  data        : HashMap<u8, VecDeque<PAMoniData>>,
  max_size    : usize,
}

//impl<T: std::marker::Copy + HasBoardId> Series<T> for PAMoniDataSeries {
impl Series<PAMoniData> for PAMoniDataSeries {

  fn get_data(&self) -> &HashMap<u8,VecDeque<PAMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<PAMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}

impl PAMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<PAMoniData>>::new(),
      max_size : 10000,
    }
  }
  
  pub fn keys(&self) -> Vec<String> {
    let keys = vec![String::from("board_id"),
                    String::from("temp_1"),
                    String::from("temp_2"),
                    String::from("temp_3"),
                    String::from("temp_4"),
                    String::from("temp_5"),
                    String::from("temp_6"),
                    String::from("temp_7"),
                    String::from("temp_8"),
                    String::from("temp_9"),
                    String::from("temp_10"),
                    String::from("temp_11"),
                    String::from("temp_12"),
                    String::from("temp_13"),
                    String::from("temp_14"),
                    String::from("temp_15"),
                    String::from("temp_16"),
                    String::from("bias_1"),
                    String::from("bias_2"),
                    String::from("bias_3"),
                    String::from("bias_4"),
                    String::from("bias_5"),
                    String::from("bias_6"),
                    String::from("bias_7"),
                    String::from("bias_8"),
                    String::from("bias_9"),
                    String::from("bias_10"),
                    String::from("bias_11"),
                    String::from("bias_12"),
                    String::from("bias_13"),
                    String::from("bias_14"),
                    String::from("bias_15"),
                    String::from("bias_16"),
    ];
    keys
  }  
  
  pub fn get_var_for_rb(&self, varname : &str,  rb_id : u8) -> Option<Vec<f32>> {
    if !self.data.contains_key(&rb_id) {
      return None;
    }
    let var = self.get_var(varname);
    if !var.contains_key(&rb_id) {
      return None; 
    }
    // FIXME This needs to be returning a reference,
    // not cloning
    Some(var[&rb_id].clone())
  }
  
  /// Get a HashMap for all boards for a specific variable
  ///
  /// See .keys() for a list of variables
  pub fn get_var(&self, varname : &str) -> HashMap<u8, Vec<f32>> {
    let mut values = HashMap::<u8, Vec<f32>>::new();
    match varname {
      "temp_1"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[0] as f32);
              }
            }
          }
        }
      },     
      "temp_2"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[1] as f32);
              }
            }
          }
        }
      },     
      "temp_3"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[2] as f32);
              }
            }
          }
        }
      },     
      "temp_4"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[3] as f32);
              }
            }
          }
        }
      },     
      "temp_5"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[4] as f32);
              }
            }
          }
        }
      },     
      "temp_6"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[5] as f32);
              }
            }
          }
        }
      },     
      "temp_7"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[6] as f32);
              }
            }
          }
        }
      },     
      "temp_8"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[7] as f32);
              }
            }
          }
        }
      },     
      "temp_9"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[8] as f32);
              }
            }
          }
        }
      },     
      "temp_10"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[9] as f32);
              }
            }
          }
        }
      },     
      "temp_11"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[10] as f32);
              }
            }
          }
        }
      },     
      "temp_12"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[11] as f32);
              }
            }
          }
        }
      },     
      "temp_13"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[12] as f32);
              }
            }
          }
        }
      },     
      "temp_14"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[13] as f32);
              }
            }
          }
        }
      },     
      "temp_15"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[14] as f32);
              }
            }
          }
        }
      },     
      "temp_16"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.temps[15] as f32);
              }
            }
          }
        }
      },     
      "bias_1"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[0] as f32);
              }
            }
          }
        }
      },     
      "bias_2"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[1] as f32);
              }
            }
          }
        }
      },     
      "bias_3"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[2] as f32);
              }
            }
          }
        }
      },     
      "bias_4"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[3] as f32);
              }
            }
          }
        }
      },     
      "bias_5"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[4] as f32);
              }
            }
          }
        }
      },     
      "bias_6"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[5] as f32);
              }
            }
          }
        }
      },     
      "bias_7"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[6] as f32);
              }
            }
          }
        }
      },     
      "bias_8"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[7] as f32);
              }
            }
          }
        }
      },     
      "bias_9"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[8] as f32);
              }
            }
          }
        }
      },     
      "bias_10"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[9] as f32);
              }
            }
          }
        }
      },     
      "bias_11"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[10] as f32);
              }
            }
          }
        }
      },     
      "bias_12"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[11] as f32);
              }
            }
          }
        }
      },     
      "bias_13"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[12] as f32);
              }
            }
          }
        }
      },     
      "bias_14"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[13] as f32);
              }
            }
          }
        }
      },     
      "bias_15"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[14] as f32);
              }
            }
          }
        }
      },     
      "bias_16"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.biases[15] as f32);
              }
            }
          }
        }
      },     
      &_                  => {
        error!("Can not get {}, since it is not a member of PAMoniData!", varname);
      }
    }
    values
  }  
  ///// Add another PAMoniData to the series
  //pub fn add(&mut self, data : PAMoniData) {
  //  if !self.data.contains_key(&data.board_id) {
  //    self.data.insert(data.board_id, VecDeque::<PAMoniData>::new());
  //  } 
  //  self.data.get_mut(&data.board_id).unwrap().push_back(data);
  //  if self.data.get_mut(&data.board_id).unwrap().len() > self.max_size {
  //    error!("The queue is too large, returning the first element! If you need a larger series size, set the max_size field");
  //    self.data.get_mut(&data.board_id).unwrap().pop_front();
  //  }
  //}
  //
  //pub fn get_last_moni(&self, board_id : u8) -> Option<PAMoniData> {
  //  let size = self.data.get(&board_id)?.len();
  //  Some(self.data.get(&board_id).unwrap()[size - 1])
  //}
}

#[derive(Debug, Clone, PartialEq)]
pub struct PBMoniDataSeries {
  data        : HashMap<u8, VecDeque<PBMoniData>>,
  max_size    : usize,
}

impl PBMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<PBMoniData>>::new(),
      max_size : 10000,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LTBMoniDataSeries {
  data        : HashMap<u8, VecDeque<LTBMoniData>>,
  max_size    : usize,
}

impl LTBMoniDataSeries {
  
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<LTBMoniData>>::new(),
      max_size : 10000,
    }
  }
  
  pub fn keys(&self) -> Vec<String> {
    let keys = vec![String::from("board_id"),
                    String::from("trenz_temp"),
                    String::from("ltb_temp"),
                    String::from("thresh_hit"),
                    String::from("thresh_beta"),
                    String::from("thresh_veto")
    ];
    keys
  }  

  /// Add another LTBMoniData to the series
  pub fn add(&mut self, data : LTBMoniData) {
    if !self.data.contains_key(&data.board_id) {
      self.data.insert(data.board_id, VecDeque::<LTBMoniData>::new());
    } 
    self.data.get_mut(&data.board_id).unwrap().push_back(data);
    if self.data.get_mut(&data.board_id).unwrap().len() > self.max_size {
      error!("The queue is too large, returning the first element! If you need a larger series size, set the max_size field");
      self.data.get_mut(&data.board_id).unwrap().pop_front();
    }
  }

  pub fn get_last_moni(&self, board_id : u8) -> Option<LTBMoniData> {
    let size = self.data.get(&board_id)?.len();
    Some(self.data.get(&board_id).unwrap()[size - 1])
  }
  
  /// Get a HashMap for all boards for a specific variable
  ///
  /// See .keys() for a list of variables
  pub fn get_var(&self, varname : &str) -> HashMap<u8, Vec<f32>> {
    let mut values = HashMap::<u8, Vec<f32>>::new();
    match varname {
      "trenz_temp"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.trenz_temp as f32);
              }
            }
          }
        }
      },     
      "ltb_temp"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.ltb_temp as f32);
              }
            }
          }
        }
      },     
      "thresh_hit"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.thresh[0] as f32);
              }
            }
          }
        }
      },     
      "thresh_beta"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.thresh[1] as f32);
              }
            }
          }
        }
      },     
      "thresh_veto"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.thresh[2] as f32);
              }
            }
          }
        }
      },
      &_                  => {
        error!("Can not get {}, since it is not a member of LTBMoniData!", varname);
      }
    }
    values
  }
  
  pub fn get_var_for_board(&self, varname : &str,  board_id : u8) -> Option<Vec<f32>> {
    if !self.data.contains_key(&board_id) {
      return None;
    }
    let var = self.get_var(varname);
    if !var.contains_key(&board_id) {
      return None; 
    }
    // FIXME This needs to be returning a reference,
    // not cloning
    Some(var[&board_id].clone())
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RBMoniDataSeries {
  data        : HashMap<u8, VecDeque<RBMoniData>>, 
  cache       : HashMap<u8, Vec<f32>>,
  cache_valid : bool,
  max_size    : usize,
}

impl RBMoniDataSeries {
  
  pub fn new() -> Self {
    Self {
      data        : HashMap::<u8, VecDeque<RBMoniData>>::new(),
      cache       : HashMap::<u8, Vec<f32>>::new(),
      cache_valid : false,
      // if the queue gets larger than this size,
      // automatically drop the first event
      max_size    : 10000,
    }
  }

  /// Get all variable names
  pub fn keys(&self) -> Vec<String> {
    let keys = vec![String::from("rate"),
                    String::from("tmp_drs"),
                    String::from("tmp_clk"),      
                    String::from("tmp_adc"),      
                    String::from("tmp_zynq"),     
                    String::from("tmp_lis3mdltr"),
                    String::from("tmp_bm280"),    
                    String::from("pressure"),     
                    String::from("humidity"),     
                    String::from("mag_x"),        
                    String::from("mag_y"),        
                    String::from("mag_z"),   
                    String::from("mag_tot"),   
                    String::from("drs_dvdd_voltage"),   
                    String::from("drs_dvdd_current"),   
                    String::from("drs_dvdd_power"),   
                    String::from("p3v3_voltage"), 
                    String::from("p3v3_current"), 
                    String::from("p3v3_power"),   
                    String::from("zynq_voltage"), 
                    String::from("zynq_current"), 
                    String::from("zynq_power"),   
                    String::from("p3v5_voltage"), 
                    String::from("p3v5_current"), 
                    String::from("p3v5_power"),   
                    String::from("adc_dvdd_voltage"),   
                    String::from("adc_dvdd_current"),   
                    String::from("adc_dvdd_power"),   
                    String::from("adc_avdd_voltage"),   
                    String::from("adc_avdd_current"),   
                    String::from("adc_avdd_power"),   
                    String::from("drs_avdd_voltage"),   
                    String::from("drs_avdd_current"),   
                    String::from("drs_avdd_power"),   
                    String::from("n1v5_voltage"),   
                    String::from("n1v5_current"),   
                    String::from("n1v5_power"),   
    ];
    keys
  }

  /// Add another RBMoniData to the series
  pub fn add(&mut self, data : RBMoniData) {
    if !self.data.contains_key(&data.board_id) {
      self.data.insert(data.board_id, VecDeque::<RBMoniData>::new());
    } 
    self.data.get_mut(&data.board_id).unwrap().push_back(data);
    if self.data.get_mut(&data.board_id).unwrap().len() > self.max_size {
      error!("The queue is too large, returning the first element! If you need a larger series size, set the max_size field");
      self.data.get_mut(&data.board_id).unwrap().pop_front();
    }
  }

  pub fn get_last_moni(&self, rb_id : u8) -> Option<RBMoniData> {
    let size = self.data.get(&rb_id)?.len();
    Some(self.data.get(&rb_id).unwrap()[size - 1])
  }

  /// 
  //pub fn get_variable_for_rb(&self, rb_id : u8, varname : &str) -> Vec<f32> {
  //  // FIXME
  //  let values = self.get_variables(varname);
  //  return values[&rb_id]; 
  //}

  //fn cache_it(&mut self) {
  //  let mut values = HashMap::<u8, Vec<f32>>::new();
  //  for varname in self.keys() {
  //    
  //  }
  //}

  pub fn get_var_for_rb(&self, varname : &str,  rb_id : u8) -> Option<Vec<f32>> {
    if !self.data.contains_key(&rb_id) {
      return None;
    }
    let var = self.get_var(varname);
    if !var.contains_key(&rb_id) {
      return None; 
    }
    // FIXME This needs to be returning a reference,
    // not cloning
    Some(var[&rb_id].clone())
  }


  /// Get a HashMap for all boards for a specific variable
  ///
  /// See .keys() for a list of variables
  pub fn get_var(&self, varname : &str) -> HashMap<u8, Vec<f32>> {
    let mut values = HashMap::<u8, Vec<f32>>::new();
    match varname {
      "rate"              => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.rate as f32);
              }
            }
          }
        }
      },     
      "tmp_drs"           => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.tmp_drs);
              }
            }
          }
        }
      },  
      "tmp_clk"           => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.tmp_clk);
              }
            }
          }
        }
      },  
      "tmp_adc"           => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.tmp_adc);
              }
            }
          }
        }
      },  
      "tmp_zynq"          => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.tmp_zynq);
              }
            }
          }
        }
      },  
      "tmp_lis3mdltr"     => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.tmp_lis3mdltr);
              }
            }
          }
        }
      },  
      "tmp_bm280"         => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.tmp_bm280);
              }
            }
          }
        }
      },  
      "pressure"          => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.pressure);
              }
            }
          }
        }
      },  
      "humidity"          => {  
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.humidity);
              }
            }
          }
        }
      },
      "mag_x"             => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.mag_x);
              }
            }
          }
        }
      },  
      "mag_y"             => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.mag_y);
              }
            }
          }
        }
      },  
      "mag_z"             => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.mag_z);
              }
            }
          }
        }
      },  
      "mag_tot"             => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.get_mag_tot());
              }
            }
          }
        }
      },  
      "drs_dvdd_voltage"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.drs_dvdd_voltage);
              }
            }
          }
        }
      },  
      "drs_dvdd_current"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.drs_dvdd_current);
              }
            }
          }
        }
      },  
      "drs_dvdd_power"    => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.drs_dvdd_power);
              }
            }
          }
        }
      },  
      "p3v3_voltage"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.p3v3_voltage);
              }
            }
          }
        }
      },  
      "p3v3_current"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.p3v3_current);
              }
            }
          }
        }
      },  
      "p3v3_power"        => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.p3v3_power);
              }
            }
          }
        }
      },  
      "zynq_voltage"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.zynq_voltage);
              }
            }
          }
        }
      },  
      "zynq_current"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.zynq_current);
              }
            }
          }
        }
      },  
      "zynq_power"        => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.zynq_power);
              }
            }
          }
        }
      },  
      "p3v5_voltage"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.p3v5_voltage);
              }
            }
          }
        }
      },  
      "p3v5_current"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.p3v5_current);
              }
            }
          }
        }
      },  
      "p3v5_power"        => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.p3v5_power);
              }
            }
          }
        }
      },  
      "adc_dvdd_voltage"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.adc_dvdd_voltage);
              }
            }
          }
        }
      },  
      "adc_dvdd_current"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.adc_dvdd_current);
              }
            }
          }
        }
      },  
      "adc_dvdd_power"    => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.adc_dvdd_power);
              }
            }
          }
        }
      },  
      "adc_avdd_voltage"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.adc_avdd_voltage);
              }
            }
          }
        }
      },  
      "adc_avdd_current"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.adc_avdd_current);
              }
            }
          }
        }
      },  
      "adc_avdd_power"    => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.adc_avdd_power);
              }
            }
          }
        }
      },  
      "drs_avdd_voltage"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.drs_avdd_voltage);
              }
            }
          }
        }
      },  
      "drs_avdd_current"  => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.drs_avdd_current);
              }
            }
          }
        }
      },  
      "drs_avdd_power"    => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.drs_avdd_power);
              }
            }
          }
        }
      },  
      "n1v5_voltage"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.n1v5_voltage);
              }
            }
          }
        }
      },  
      "n1v5_current"      => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.n1v5_current);
              }
            }
          }
        }
      },  
      "n1v5_power"        => {
        for k in self.data.keys() {
          values.insert(*k, Vec::<f32>::new());
          match self.data.get(k) {
            None => (),
            Some(vec_moni) => {
              for moni in vec_moni {
                values.get_mut(k).unwrap().push(moni.n1v5_power);
              }
            }
          }
        }
      },  
      &_                  => {
        error!("Can not get {}, since it is not a member of RBMoniData!", varname);
      }
    }
    values
  }
}

