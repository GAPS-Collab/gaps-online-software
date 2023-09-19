//! Set the DRS4 idle register and then watch the buffers

extern crate liftof_rb;

use liftof_rb::api::*;
use liftof_rb::control::*;

fn main() {

  match reset_dma() {
    Ok(_) => (),
    Err(err) => println!("Unable to reset DMA, error {err}")
  }
  run_check();
}

