//use std::collections::VecDeque;
use std::collections::HashMap;
//use std::path::Path;
pub mod dataclasses;
pub mod master_trigger;

use pyo3::prelude::*;

extern crate pyo3_log;
extern crate comfy_table;

use rpy_tof_dataclasses::dataclasses::{
    PyMasterTriggerEvent,
    PyRBEvent,
};

use crate::dataclasses::PyIPBus;

use crate::master_trigger::{
    PyMasterTrigger,
};

use tof_dataclasses::analysis::{
    calc_edep_simple
};

use tof_dataclasses::database::{
    RAT,
    DSICard,
    Paddle,
    MTBChannel,
    LocalTriggerBoard,
    ReadoutBoard,
    Panel,
    connect_to_db,
};

use tof_dataclasses::events::{
    //RBEvent, 
    TofEvent
};

use tof_dataclasses::packets::PacketType;
use tof_dataclasses::io::TofPacketReader;
//use tof_dataclasses::serialization::Serialization;

use liftof_lib::waveform_analysis;
use liftof_lib::settings::AnalysisEngineSettings;

#[pyfunction]
#[pyo3(name="test_db")]
pub fn test_db() {
  let mut conn = connect_to_db(String::from("/srv/gaps/gaps-online-software/gaps-db/gaps_db/gaps_flight.db")).unwrap();
  let rats = RAT::all(&mut conn).unwrap();
  for r in rats {
    println!("{}", r);
  }
  let dsis = DSICard::all(&mut conn).unwrap();
  for dsi in dsis {
    println!("{}", dsi);
  }
  let paddles = Paddle::all(&mut conn).unwrap();
  for pdl in paddles {
    println!("{}", pdl);
  }
  let mtbch = MTBChannel::all(&mut conn).unwrap();
  for chnl in mtbch {
    println!("{}", chnl);
  }
  let ltbs = LocalTriggerBoard::all(&mut conn).unwrap();
  for ltb in ltbs {
    println!("{}", ltb);
  }
  let rbs = ReadoutBoard::all(&mut conn).unwrap();
  for rb in rbs {
    println!("{}", rb);
  }
  let panels = Panel::all(&mut conn).unwrap();
  for pnl in panels {
    println!("{}", pnl);
  }
}


#[pyfunction]
#[pyo3(name="calc_edep_simple")]
pub fn wrap_calc_edep_simple(peak_voltage : f32) -> f32 {
  calc_edep_simple(peak_voltage)
}


#[pyfunction]
#[pyo3(name = "test_waveform_analysis")]
fn test_waveform_analysis(filename : String) -> PyRBEvent {
  let mut settings   = AnalysisEngineSettings::new();
  settings.find_pks_t_start  = 60.0;
  settings.find_pks_t_window = 300.0;
  settings.min_peak_size     = 10;
  //let rb         = ReadoutBoard::new();
  let pth        = String::from("/srv/gaps/gaps-online-software/gaps-db/gaps_db/gaps_flight.db");
  let mut conn   = connect_to_db(pth).unwrap();
  let rbs        = ReadoutBoard::all(&mut conn).unwrap();
  let mut rb_map = HashMap::<u8, ReadoutBoard>::new();
  for mut rb in rbs {
    rb.calib_file_path = String::from("/data0/gaps/nevis/calib/latest/"); 
    match rb.load_latest_calibration() {
      Err(_err) => {
        // FIXME - come up with error thing
        //error!("Unable to laod calibration data for ReadoutBoards!");
      }
      Ok(_) => ()
    }
    rb_map.insert(rb.rb_id, rb);
  }
  let mut reader  = TofPacketReader::new(filename);
  let mut py_rbev = PyRBEvent::new();
  //let mut n_ev    = 0u32;
  loop {
    match reader.next()  {
      Some(tp) => {
        match tp.packet_type {
          PacketType::TofEvent => {
            match tp.unpack::<TofEvent>() {
            //match TofEvent::from_tofpacket(&tp) {
              Err(_err) => {
                //error!("Unable to unpack TofEvent!");
              },
              Ok(te) => {
                //println!("{}", te);
                if te.rb_events.is_empty() {
                  continue;
                }
                for mut rbev in te.rb_events {
                  let rb_id = rbev.header.rb_id;
                  //println!("{}", rbev); 
                  py_rbev.set_event(rbev.clone());
                  match waveform_analysis(
                    &mut rbev,
                    &rb_map[&rb_id],
                    settings.clone()
                  ) {
                    // FIXME!
                    Err(_err) => (),
                    Ok(_)     => ()
                  }
                  for h in rbev.hits {
                    println!("{}", h);
                  }
                  return py_rbev;
                  //break;
                }
              }
            }
          },
          _ => ()      
        }
      },
      None => {
        break;
      }
    }
  }
  return py_rbev;
}



/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[pymodule]
#[pyo3(name = "rust_dataclasses")]
fn rust_dataclasses<'_py>(m : &Bound<'_py, PyModule>) -> PyResult<()> { 
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(test_waveform_analysis,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_calc_edep_simple,m)?)?;
    m.add_function(wrap_pyfunction!(test_db,m)?)?;
    m.add_class::<PyIPBus>()?;
    m.add_class::<PyMasterTrigger>()?;
    Ok(())
}
