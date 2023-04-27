#[cfg(test)]
pub mod tests {

  extern crate rand;
  use rand::Rng;
  use std::path::Path;
  use tof_dataclasses::events::RBBinaryDump;
  use tof_dataclasses::constants::{NWORDS, NCHN, MAX_NUM_PEAKS};
  use tof_dataclasses::serialization::Serialization;
  use tof_dataclasses::FromRandom;
  use tof_dataclasses::io::read_file;

  #[test]
  fn read_file_test_for_rbbinarydump() {
    let stream = read_file(&Path::new("test-data/tof-rb01.robin")).unwrap();
    let size = stream.len();
    let mut pos = 0usize;
    let mut events = Vec::<RBBinaryDump>::new(); 
    //let mut n_events = (size / 15830) as usize;
    let mut event  = RBBinaryDump::new();
    let mut n_broken = 0;
    while pos <= size {
      println!("pos {pos}");
      match RBBinaryDump::from_bytestream(&stream,&mut pos) {
        Ok(event) => { events.push(event);},
        Err(err)  => {
          println!("error decoding RBBinaryDump, err {err}");
          n_broken += 1;
          pos += 1;
        }
      }
      println!("We saw {} broken events", n_broken);
    }
  }

  #[test]
  fn serialization_circle_test_for_rbbinarydump() {
    // try this 100 times
    for n in 0..100 {
      println!("Iter {n}");
      let rb_bin       = RBBinaryDump::from_random();
      //println!("RBBinaryDump {}", rb_bin);
      let rb_bin_ser   = rb_bin.to_bytestream();
      //println!("Found stream len {}", rb_bin_ser.len());
      let mut pos = 0usize;
      let rb_bin_deser = RBBinaryDump::from_bytestream(&rb_bin_ser, &mut pos);
      //println!("After ser/deser {}", rb_bin_deser.as_ref().unwrap());
      assert_eq!(rb_bin_deser.unwrap(), rb_bin);
    }
  }

  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}

