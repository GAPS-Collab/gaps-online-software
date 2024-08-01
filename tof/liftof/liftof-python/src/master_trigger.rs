use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
//use numpy::PyArray1;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

use tof_dataclasses::ipbus::IPBus;
use tof_dataclasses::events::MasterTriggerEvent;

use liftof_lib::master_trigger::registers::*;
use liftof_lib::master_trigger as mt_api;

use crate::dataclasses::{
    PyMasterTriggerEvent,
};

#[pyclass]
#[pyo3(name = "MasterTrigger")]
pub struct PyMasterTrigger {
  ipbus : IPBus,
}

#[pymethods]
impl PyMasterTrigger {
  #[new]
  fn new(target_address : String) -> Self {
    let ipbus = IPBus::new(target_address).expect("Unable to connect to {target_address}");
    Self {
      ipbus : ipbus,
    }
  }

  fn reset_daq(&mut self) -> PyResult<()>{
    match self.ipbus.write(0x10,1) {
      Ok(result) => {
        return Ok(result); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_expected_pid(&mut self) -> PyResult<u16> {
    match self.ipbus.get_target_next_expected_packet_id(){
      Ok(result) => {
        return Ok(result); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn realign_packet_id(&mut self) -> PyResult<()> {
    match self.ipbus.realign_packet_id() {
      Ok(_) => {
        return Ok(()); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_packet_id(&mut self, pid : u16) {
    self.ipbus.pid = pid;
  }

  fn get_packet_id(&mut self) -> u16 {
    self.ipbus.pid
  }
 
  /// Get the global trigger rate in Hz
  fn get_rate(&mut self) -> PyResult<u32> {
    match TRIGGER_RATE.get(&mut self.ipbus) {
      Ok(rate) => {
        return Ok(rate);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// Check if the TIU emulation mode is on
  ///
  /// # Arguments:
  ///
  /// * bus : IPBus 
  fn get_tiu_emulation_mode(&mut self) -> PyResult<u32> {
    match TIU_EMULATION_MODE.get(&mut self.ipbus) {
      Ok(mode) => {
        return Ok(mode);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_tiu_emulation_mode(&mut self, value : u32) -> PyResult<()> {
    match TIU_EMULATION_MODE.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_track_trigger_is_global(&mut self) -> PyResult<()> {
    match TRACK_CENTRAL_IS_GLOBAL.set(&mut self.ipbus, 1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// Get the lost global trigger rate in Hz
  ///
  /// This is the rate of triggers which got 
  /// dropped due to TIU BUSY signal
  fn get_lost_rate(&mut self) -> PyResult<u32> {
    match LOST_TRIGGER_RATE.get(&mut self.ipbus) {
      Ok(rate) => {
        return Ok(rate);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_ltb_event_cnts(&mut self) -> PyResult<HashMap<u8, u32>> {
    let registers = [LT0, LT1, LT2, LT3, LT4, LT5, LT6, LT7, LT8, LT9,
                     LT10, LT11, LT12, LT13, LT14, LT15, LT16, LT17, LT18, LT19,
                     LT20, LT21, LT22, LT23, LT24];
    let mut counters = HashMap::<u8, u32>::new();
    for (k,reg) in registers.iter().enumerate() {
      match reg.get(&mut self.ipbus) {
        Err(err) => {
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(cnt) => {
          counters.insert(k as u8, cnt);
        }
      }
    }
    // print a table
    let mut table = Table::new();
    table
      .load_preset(UTF8_FULL)
      .apply_modifier(UTF8_ROUND_CORNERS)
      .set_content_arrangement(ContentArrangement::Dynamic)
      .set_width(80)
      .set_header(vec!["LT 0", "LT 1", "LT 2", "LT 3", "LT 4"])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&0]))),
          Cell::new(&(format!("{}", counters[&1]))),
          Cell::new(&(format!("{}", counters[&2]))),
          Cell::new(&(format!("{}", counters[&3]))),
          Cell::new(&(format!("{}", counters[&4]))),
          //Cell::new("Center aligned").set_alignment(CellAlignment::Center),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 5")),
          Cell::new(String::from("LT 6")),
          Cell::new(String::from("LT 7")),
          Cell::new(String::from("LT 8")),
          Cell::new(String::from("LT 9")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&5]))),
          Cell::new(&(format!("{}", counters[&6]))),
          Cell::new(&(format!("{}", counters[&7]))),
          Cell::new(&(format!("{}", counters[&8]))),
          Cell::new(&(format!("{}", counters[&9]))),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 10")),
          Cell::new(String::from("LT 11")),
          Cell::new(String::from("LT 12")),
          Cell::new(String::from("LT 13")),
          Cell::new(String::from("LT 14")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&10]))),
          Cell::new(&(format!("{}", counters[&11]))),
          Cell::new(&(format!("{}", counters[&12]))),
          Cell::new(&(format!("{}", counters[&13]))),
          Cell::new(&(format!("{}", counters[&14]))),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 15")),
          Cell::new(String::from("LT 16")),
          Cell::new(String::from("LT 17")),
          Cell::new(String::from("LT 18")),
          Cell::new(String::from("LT 19")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&15]))),
          Cell::new(&(format!("{}", counters[&16]))),
          Cell::new(&(format!("{}", counters[&17]))),
          Cell::new(&(format!("{}", counters[&18]))),
          Cell::new(&(format!("{}", counters[&19]))),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 20")),
          Cell::new(String::from("LT 21")),
          Cell::new(String::from("LT 22")),
          Cell::new(String::from("LT 23")),
          Cell::new(String::from("LT 24")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&20]))),
          Cell::new(&(format!("{}", counters[&21]))),
          Cell::new(&(format!("{}", counters[&22]))),
          Cell::new(&(format!("{}", counters[&23]))),
          Cell::new(&(format!("{}", counters[&24]))),
      ]);

    // Set the default alignment for the third column to right
    let column = table.column_mut(2).expect("Our table has three columns");
    column.set_cell_alignment(CellAlignment::Right);
    println!("{table}");
    Ok(counters)
  }
  
  /// Readout the RB event counter registers
  fn get_rb_event_cnts(&mut self) -> PyResult<HashMap<u8, u8>> {
    let registers = [RB0_CNTS, RB1_CNTS, RB2_CNTS, RB3_CNTS, RB4_CNTS,
                     RB5_CNTS, RB6_CNTS, RB7_CNTS, RB8_CNTS, RB9_CNTS,
                     RB10_CNTS, RB11_CNTS, RB12_CNTS, RB13_CNTS, RB14_CNTS,
                     RB15_CNTS, RB16_CNTS, RB17_CNTS, RB18_CNTS, RB19_CNTS,
                     RB20_CNTS, RB21_CNTS, RB22_CNTS, RB23_CNTS, RB24_CNTS,
                     RB25_CNTS, RB26_CNTS, RB27_CNTS, RB28_CNTS, RB29_CNTS,
                     RB30_CNTS, RB31_CNTS, RB32_CNTS, RB33_CNTS, RB34_CNTS,
                     RB35_CNTS, RB36_CNTS, RB37_CNTS, RB38_CNTS, RB39_CNTS,
                     RB40_CNTS, RB41_CNTS, RB42_CNTS, RB43_CNTS, RB44_CNTS,
                     RB45_CNTS, RB46_CNTS, RB47_CNTS, RB48_CNTS, RB49_CNTS];
    let mut counters = HashMap::<u8, u8>::new();
    for (k,reg) in registers.iter().enumerate() {
      match reg.get(&mut self.ipbus) {
        Err(err) => {
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(cnt) => {
          counters.insert(k as u8, cnt as u8);
        }
      }
    }
    let mut table = Table::new();
    table
      .load_preset(UTF8_FULL)
      .apply_modifier(UTF8_ROUND_CORNERS)
      .set_content_arrangement(ContentArrangement::Dynamic)
      .set_width(60)
      .set_header(vec!["RB 0", "RB 1", "RB 2", "RB 3", "RB 4"])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&0]))),
          Cell::new(&(format!("{}", counters[&1]))),
          Cell::new(&(format!("{}", counters[&2]))),
          Cell::new(&(format!("{}", counters[&3]))),
          Cell::new(&(format!("{}", counters[&4]))),
          //Cell::new("Center aligned").set_alignment(CellAlignment::Center),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 5")),
          Cell::new(String::from("RB 6")),
          Cell::new(String::from("RB 7")),
          Cell::new(String::from("RB 8")),
          Cell::new(String::from("RB 9")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&5]))),
          Cell::new(&(format!("{}", counters[&6]))),
          Cell::new(&(format!("{}", counters[&7]))),
          Cell::new(&(format!("{}", counters[&8]))),
          Cell::new(&(format!("{}", counters[&9]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 10")),
          Cell::new(String::from("RB 11")),
          Cell::new(String::from("RB 12")),
          Cell::new(String::from("RB 13")),
          Cell::new(String::from("RB 14")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&10]))),
          Cell::new(&(format!("{}", counters[&11]))),
          Cell::new(&(format!("{}", counters[&12]))),
          Cell::new(&(format!("{}", counters[&13]))),
          Cell::new(&(format!("{}", counters[&14]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 15")),
          Cell::new(String::from("RB 16")),
          Cell::new(String::from("RB 17")),
          Cell::new(String::from("RB 18")),
          Cell::new(String::from("RB 19")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&15]))),
          Cell::new(&(format!("{}", counters[&16]))),
          Cell::new(&(format!("{}", counters[&17]))),
          Cell::new(&(format!("{}", counters[&18]))),
          Cell::new(&(format!("{}", counters[&19]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 20")),
          Cell::new(String::from("RB 21")),
          Cell::new(String::from("RB 22")),
          Cell::new(String::from("RB 23")),
          Cell::new(String::from("RB 24")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&20]))),
          Cell::new(&(format!("{}", counters[&21]))),
          Cell::new(&(format!("{}", counters[&22]))),
          Cell::new(&(format!("{}", counters[&23]))),
          Cell::new(&(format!("{}", counters[&24]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 25")),
          Cell::new(String::from("RB 26")),
          Cell::new(String::from("RB 27")),
          Cell::new(String::from("RB 28")),
          Cell::new(String::from("RB 29")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&25]))),
          Cell::new(&(format!("{}", counters[&26]))),
          Cell::new(&(format!("{}", counters[&27]))),
          Cell::new(&(format!("{}", counters[&28]))),
          Cell::new(&(format!("{}", counters[&29]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 30")),
          Cell::new(String::from("RB 31")),
          Cell::new(String::from("RB 32")),
          Cell::new(String::from("RB 33")),
          Cell::new(String::from("RB 34")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&30]))),
          Cell::new(&(format!("{}", counters[&31]))),
          Cell::new(&(format!("{}", counters[&32]))),
          Cell::new(&(format!("{}", counters[&33]))),
          Cell::new(&(format!("{}", counters[&34]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 35")),
          Cell::new(String::from("RB 36")),
          Cell::new(String::from("RB 37")),
          Cell::new(String::from("RB 38")),
          Cell::new(String::from("RB 39")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&35]))),
          Cell::new(&(format!("{}", counters[&36]))),
          Cell::new(&(format!("{}", counters[&37]))),
          Cell::new(&(format!("{}", counters[&38]))),
          Cell::new(&(format!("{}", counters[&39]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 40")),
          Cell::new(String::from("RB 41")),
          Cell::new(String::from("RB 42")),
          Cell::new(String::from("RB 43")),
          Cell::new(String::from("RB 44")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&40]))),
          Cell::new(&(format!("{}", counters[&41]))),
          Cell::new(&(format!("{}", counters[&42]))),
          Cell::new(&(format!("{}", counters[&43]))),
          Cell::new(&(format!("{}", counters[&44]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 45")),
          Cell::new(String::from("RB 46")),
          Cell::new(String::from("RB 47")),
          Cell::new(String::from("RB 48")),
          Cell::new(String::from("RB 49")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&45]))),
          Cell::new(&(format!("{}", counters[&46]))),
          Cell::new(&(format!("{}", counters[&47]))),
          Cell::new(&(format!("{}", counters[&48]))),
          Cell::new(&(format!("{}", counters[&49]))),
      ]);

    // Set the default alignment for the third column to right
    let column = table.column_mut(2).expect("Our table has three columns");
    column.set_cell_alignment(CellAlignment::Right);
    println!("{table}");
    Ok(counters)
  }
  
  /// Reset all the RB counters
  fn reset_rb_counters(&mut self) -> PyResult<()> {
    println!("{}", RB_CNTS_RESET);
    match RB_CNTS_RESET.set(&mut self.ipbus, 1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// Reset all the LTB counters
  fn reset_ltb_counters(&mut self) -> PyResult<()> {
    match LT_HIT_CNT_RESET.set(&mut self.ipbus, 1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  /// Set a channel mask for a LTB. 
  ///
  /// # Arguments
  /// * lt_link : 0-24, dsi/j connection of the LTB on the MTB
  /// * mask    : bitmask 1 = ch0 2 = ch1, etc. setting a channel
  ///             to 1 will DISABLE the channel!
  fn set_ltb_ch_mask(&mut self, lt_link : u8, mask : u8) -> PyResult<()> {
    let registers = [LT0_CHMASK, LT1_CHMASK, LT2_CHMASK, LT3_CHMASK, LT4_CHMASK,
                     LT5_CHMASK, LT6_CHMASK, LT7_CHMASK, LT8_CHMASK, LT9_CHMASK,
                     LT10_CHMASK, LT11_CHMASK, LT12_CHMASK, LT13_CHMASK, LT14_CHMASK,
                     LT15_CHMASK, LT16_CHMASK, LT17_CHMASK, LT18_CHMASK, LT19_CHMASK,
                     LT20_CHMASK, LT21_CHMASK, LT22_CHMASK, LT23_CHMASK, LT24_CHMASK];
    if lt_link as usize > registers.len() {
      return Err(PyValueError::new_err(String::from("Mask has to be in range 0-24!")));
    }

    match registers[lt_link as usize].set(&mut self.ipbus, mask as u32) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  
  fn set_trace_suppression(&mut self, trace_sup : bool) -> PyResult<()> {
    let read_all_rb : u32;
    if trace_sup {
      read_all_rb = 0;
    } else {
      read_all_rb = 1;
    }
    match RB_READ_ALL_CHANNELS.set(&mut self.ipbus, read_all_rb) {
      Ok(_)  => {
        Ok(())
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_trace_suppression(&mut self) -> PyResult<u32> {
    match RB_READ_ALL_CHANNELS.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_total_tof_thresh(&mut self, value : u32) -> PyResult<()> {
    match TOTAL_TOF_THRESH.set(&mut self.ipbus, value) {
      Ok(_)  => {
        Ok(())
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_total_tof_thresh(&mut self) -> PyResult<u32> {
    match TOTAL_TOF_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_inner_tof_thresh(&mut self, value : u32) -> PyResult<()> {
    match INNER_TOF_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_inner_tof_thresh(&mut self) -> PyResult<u32> {
    match INNER_TOF_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_outer_tof_thresh(&mut self, value : u32) -> PyResult<()> {
    match OUTER_TOF_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_outer_tof_thresh(&mut self) -> PyResult<u32> {
    match OUTER_TOF_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_side_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_SIDE_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_side_thresh(&mut self) -> PyResult<u32> {
    match CUBE_SIDE_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_top_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_TOP_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_top_thresh(&mut self) -> PyResult<u32> {
    match CUBE_TOP_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_bot_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_BOT_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_bot_thresh(&mut self) -> PyResult<u32> {
    match CUBE_BOT_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_corner_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_CORNER_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_corner_thresh(&mut self) -> PyResult<u32> {
    match CUBE_CORNER_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 
  fn set_umbrella_thresh(&mut self, value : u32) -> PyResult<()> {
    match UMBRELLA_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_umbrella_thresh(&mut self) -> PyResult<u32> {
    match UMBRELLA_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_umbrella_center_thresh(&mut self, value : u32) -> PyResult<()> {
    match UMBRELLA_CENTER_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_umbrella_center_thresh(&mut self) -> PyResult<u32> {
    match UMBRELLA_CENTER_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cortina_thresh(&mut self, value : u32) -> PyResult<()> {
    match CORTINA_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_cortina_thresh(&mut self) -> PyResult<u32> {
    match CORTINA_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_configurable_trigger(&mut self, value : u32) -> PyResult<()> {
    match CONFIGURABLE_TRIGGER_EN.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_configurable_trigger(&mut self) -> PyResult<u32> {
    match CONFIGURABLE_TRIGGER_EN.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_any_trigger(&mut self, prescale : u32) -> PyResult<()> {
    match ANY_TRIG_PRESCALE.set(&mut self.ipbus, prescale) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_track_trigger(&mut self, prescale : u32) -> PyResult<()> {
    match TRACK_TRIG_PRESCALE.set(&mut self.ipbus, prescale) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_central_track_trigger(&mut self, prescale : u32) -> PyResult<()> {
    match TRACK_CENTRAL_PRESCALE.set(&mut self.ipbus, prescale) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn use_tiu_aux_link(&mut self, use_it : bool) -> PyResult<()> {
    match mt_api::control::use_tiu_aux_link(&mut self.ipbus, use_it) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn stop_all_triggers(&mut self) -> PyResult<()> {
    match mt_api::control::unset_all_triggers(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_umbcube_trigger(&mut self) -> PyResult<()> {
    match mt_api::control::set_umbcube_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_umbcubez_trigger(&mut self) -> PyResult<()> {
    match mt_api::control::set_umbcubez_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_umbcorcube_trigger(&mut self) -> PyResult<()> {
    match mt_api::control::set_umbcorcube_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_corcubeside_trigger(&mut self) -> PyResult<()> {
    match mt_api::control::set_corcubeside_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_umb3cube_trigger(&mut self) -> PyResult<()> {
    match mt_api::control::set_umb3cube_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_tiu_busy_ignore(&mut self) -> PyResult<bool> {
    match TIU_BUSY_IGNORE.get(&mut self.ipbus) {
      Ok(bsy) => {
        let res = bsy != 0;
        return Ok(res);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_tiu_busy_ignore(&mut self, bsy : bool) -> PyResult<()> {
    match TIU_BUSY_IGNORE.set(&mut self.ipbus, bsy as u32) {
      Ok(bsy) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_event_cnt(&mut self) -> PyResult<u32> {
    match EVENT_CNT.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_event_queue_size(&mut self)
    -> PyResult<u32> {
    match EVQ_SIZE.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_event_queue_full(&mut self)
    -> PyResult<u32> {
    match EVQ_FULL.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_nevents_in_queue(&mut self) 
    -> PyResult<u32> {
    match EVQ_NUM_EVENTS.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  

  fn get_event(&mut self, read_until_footer : bool, verbose : bool, debug : bool)
    -> PyResult<PyMasterTriggerEvent> {
    let use_dbg_version = debug;
    if !use_dbg_version {
      match mt_api::get_event(&mut self.ipbus) {
        Err(err) => {
          //error!("Unable to obtain event from the MTB!");
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(mte) => {
          let mut event = PyMasterTriggerEvent::new();
          event.set_event(mte);
          Ok(event)
        }
      }
    } else {
      // This can be great for debugging. However, at some point 
      // I'd like to introduce debugging features and have all 
      // the debugging at the same place
      let mut n_daq_words : u16;
      let mut n_daq_words_actual : u16;
      loop {
        match EVQ_NUM_EVENTS.get(&mut self.ipbus) {
          Err(_err) => {
            continue;
          }
          Ok(nevents_in_q) => {
            if nevents_in_q == 0 {
              if verbose {
                println!("[MasterTrigger::get_event] => EventQueue empty!!");
              }
              return Err(PyValueError::new_err(String::from("<MasterTriggerError: EventQueueEmpty>")));
            }
          }
        }
        match self.ipbus.read(0x13) { 
          Err(_err) => {
            // A timeout does not ncecessarily mean that there 
            // is no event, it can also just mean that 
            // the rate is low.
            //trace!("Timeout in read_register for MTB! {err}");
            continue;
          },
          Ok(_n_words) => {
            n_daq_words = (_n_words >> 16) as u16;
            if _n_words == 0 {
              continue;
            }
            //trace!("Got n_daq_words {n_daq_words}");
            let rest = n_daq_words % 2;
            n_daq_words /= 2 + rest; //mtb internally operates in 16bit words, but 
            //                  //registers return 32bit words.
            
            break;
          }
        }
      }
      let mut data : Vec<u32>;
      if verbose {
        println!("[MasterTrigger::get_event] => Will query DAQ for {n_daq_words} words!");
      }
      n_daq_words_actual = n_daq_words;
      match self.ipbus.read_multiple(
                                     0x11,
                                     n_daq_words as usize,
                                     false) {
        Err(err) => {
          if verbose {
            println!("[MasterTrigger::get_event] => failed! {err}");
          }
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(_data) => {
          data = _data;
          for (i,word) in data.iter().enumerate() {
            let desc : &str;
            let desc_str : String;
            //let mut nhit_words = 0;
            match i {
              0 => desc = "HEADER",
              1 => desc = "EVENTID",
              2 => desc = "TIMESTAMP",
              3 => desc = "TIU_TIMESTAMP",
              4 => desc = "TIU_GPS32",
              5 => desc = "TIU_GPS16 + TRIG_SOURCE",
              6 => desc = "RB MASK 0",
              7 => desc = "RB MASK 1",
              8 => {
                //nhit_words = nhit_words / 2 + nhit_words % 2;
                desc_str  = format!("BOARD MASK ({} ltbs)", word.count_ones());
                desc  = &desc_str;
              },
              _ => desc = "?"
            }
            if verbose {
              println!("[MasterTrigger::get_event] => DAQ word {}    \t({:x})    \t[{}]", word, word, desc);
            }
          }
        }
      }
      if data[0] != 0xAAAAAAAA {
        if verbose {
          println!("[MasterTrigger::get_event] => Got MTB data, but the header is incorrect {}", data[0]);
        }
        return Err(PyValueError::new_err(String::from("Incorrect header value!")));
      }
      let foot_pos = (n_daq_words - 1) as usize;
      if data.len() <= foot_pos {
        if verbose {
          println!("[MasterTrigger::get_event] => Got MTB data, but the format is not correct");
        }
        return Err(PyValueError::new_err(String::from("Empty data!")));
      }
      if data[foot_pos] != 0x55555555 {
        if verbose {
          println!("[MasterTrigger::get_event] => Did not read unti footer!");
        }
        if read_until_footer {
          if verbose {
            println!("[MasterTrigger::get_event] => .. will read additional words!");
          }
          loop {
            match self.ipbus.read(0x11) {
              Err(err) => {
                if verbose {
                  println!("[MasterTrigger::get_event] => Issues reading from 0x11");
                }
                return Err(PyValueError::new_err(err.to_string()));
              },
              Ok(next_word) => {
                n_daq_words_actual += 1;
                data.push(next_word);
                if next_word == 0x55555555 {
                  break;
                }
              }
            }
          }
          if verbose {
            println!("[MasterTrigger::get_event] => We read {} additional words!", n_daq_words_actual - n_daq_words);
          }
        } else {
          if verbose {
            println!("[MasterTrigger::get_event] => Got MTB data, but the footer is incorrect {}", data[foot_pos]);
          }
          return Err(PyValueError::new_err(String::from("Footer incorrect!")));
        }
      }

      // Number of words which will be always there. 
      // Min event size is +1 word for hits
      //const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 9; 
      //let n_hit_packets = n_daq_words as u32 - MTB_DAQ_PACKET_FIXED_N_WORDS;
      //println!("We are expecting {}", n_hit_packets);
      let mut mte = MasterTriggerEvent::new();
      mte.event_id       = data[1];
      mte.timestamp      = data[2];
      mte.tiu_timestamp  = data[3];
      mte.tiu_gps32      = data[4];
      mte.tiu_gps16      = (data[5] & 0x0000ffff) as u16;
      mte.trigger_source = ((data[5] & 0xffff0000) >> 16) as u16;
      //mte.get_trigger_sources();
      let rbmask = (data[7] as u64) << 31 | data[6] as u64; 
      mte.mtb_link_mask  = rbmask;
      mte.dsi_j_mask     = data[8];
      let mut n_hit_words    = n_daq_words_actual - 9 - 2; // fixed part is 11 words
      if n_hit_words > n_daq_words_actual {
        n_hit_words = 0;
        println!("[MasterTrigger::get_event] N hit word calculation failed! fixing... {}", n_hit_words);
      }
      if verbose {
        println!("[MasterTrigger::get_event] => Will read {} hit word", n_hit_words);
      }
      for k in 1..n_hit_words+1 {
        if verbose {
          println!("[MasterTrigger::get_event] => Getting word {}", k);
        }
        let first  = (data[8 + k as usize] & 0x0000ffff) as u16;
        let second = ((data[8 + k as usize] & 0xffff0000) >> 16) as u16; 
        mte.channel_mask.push(first);
        if second != 0 {
          mte.channel_mask.push(second);
        }
      }
      if verbose {
        println!("[MasterTrigger::get_event] => Got MTE \n{}", mte);
      }
      let mut event = PyMasterTriggerEvent::new();
      event.set_event(mte);
      Ok(event)
    }
  }
}

