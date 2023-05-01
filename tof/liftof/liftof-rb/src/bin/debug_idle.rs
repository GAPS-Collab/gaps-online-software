//! Set the DRS4 idle register and then watch the buffers

extern crate liftof_rb;

use liftof_rb::api::*;
use liftof_rb::control::*;
extern crate pretty_env_logger;


fn main() {
  pretty_env_logger::init();

  match reset_dma() {
    Ok(_) => (),
    Err(err) => println!("Unable to reset DMA, error {err}")
  }
  run_check();
}

