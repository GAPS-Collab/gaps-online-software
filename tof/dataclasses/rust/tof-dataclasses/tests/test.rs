#[cfg(test)]
pub mod tests {

  extern crate rand;
  use rand::Rng;
  use std::path::Path;
  use tof_dataclasses::events::{RBBinaryDump, RBEventHeader};
  use tof_dataclasses::monitoring::RBMoniData;
  use tof_dataclasses::constants::{NWORDS, NCHN, MAX_NUM_PEAKS};
  use tof_dataclasses::serialization::Serialization;
  use tof_dataclasses::serialization::search_for_u16;
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
    let mut n_good   = 0;
    let mut block_size = 0;
    while pos <= size {
      //pos += 1;
      block_size = pos - block_size;
      //println!("SIZE {block_size}");
      //println!("POS {pos}");
      match RBBinaryDump::from_bytestream(&stream,&mut pos) {
        Ok(event) => {
          events.push(event);
          n_good += 1;
        },
        Err(err)  => {
          //println!("error decoding RBBinaryDump, err {err}");
          n_broken += 1;
          pos += 1;
          //break;
        }
      }
    }
    println!("We decoded {} events", n_good);
    println!("We saw {} broken events", n_broken);
  }

  #[test]
  fn serialization_circle_test_for_rbbinarydump() {
    // try this 100 times
    for _n in 0..100 {
      let rb_bin       = RBBinaryDump::from_random();
      //println!("RBBinaryDump {}", rb_bin);
      let rb_bin_ser   = rb_bin.to_bytestream();
      //println!("Found stream len {}", rb_bin_ser.len());
      let mut pos = 0usize;
      let rb_bin_deser = RBBinaryDump::from_bytestream(&rb_bin_ser, &mut pos);
      //println!("After ser/deser {}", rb_bin_deser.as_ref().unwrap());
      let result = rb_bin_deser.unwrap();
      assert_eq!(result.head, rb_bin.head);
      assert_eq!(result.status, rb_bin.status);
      assert_eq!(result.len, rb_bin.len);
      assert_eq!(result.roi, rb_bin.roi);
      assert_eq!(result.dna, rb_bin.dna);
      assert_eq!(result.fw_hash, rb_bin.fw_hash);
      assert_eq!(result.id, rb_bin.id);
      assert_eq!(result.ch_mask, rb_bin.ch_mask);
      assert_eq!(result.dtap0, rb_bin.dtap0);
      assert_eq!(result.dtap1, rb_bin.dtap1);
      assert_eq!(result, rb_bin);
    }
  }
  
  #[test]
  fn serialization_circle_test_for_rbmonidata() {
    // try this 100 times
    for _n in 0..100 {
      let moni       = RBMoniData::from_random();
      let moni_ser   = moni.to_bytestream();
      let mut pos = 0usize;
      println!("{:?}", moni_ser);
      let foo = search_for_u16(RBMoniData::HEAD, &moni_ser, 0); 
      println!("{:?}", foo);
      let mut moni_deser = RBMoniData::from_bytestream(&moni_ser, &mut pos);
      //println!("After ser/deser {}", rb_bin_deser.as_ref().unwrap());
      match moni_deser {
        Ok(result) => (),
        Err(err)   => println!("Can not deserialize RBMoniData! {err}")
      }
      pos = 0;
      moni_deser = RBMoniData::from_bytestream(&moni_ser, &mut pos);
      let result = moni_deser.unwrap(); 
      assert_eq!(result.board_id          ,moni.board_id           );
      assert_eq!(result.rate              ,moni.rate               );
      assert_eq!(result.tmp_drs           ,moni.tmp_drs            );
      assert_eq!(result.tmp_clk           ,moni.tmp_clk            );
      assert_eq!(result.tmp_adc           ,moni.tmp_adc            );
      assert_eq!(result.tmp_zynq          ,moni.tmp_zynq           );
      assert_eq!(result.tmp_lis3mdltr     ,moni.tmp_lis3mdltr      );
      assert_eq!(result.tmp_bm280         ,moni.tmp_bm280          );
      assert_eq!(result.pressure          ,moni.pressure           );
      assert_eq!(result.humidity          ,moni.humidity           );
      assert_eq!(result.mag_x             ,moni.mag_x              );
      assert_eq!(result.mag_y             ,moni.mag_y              );
      assert_eq!(result.mag_z             ,moni.mag_z              );
      assert_eq!(result.mag_tot           ,moni.mag_tot            );
      assert_eq!(result.drs_dvdd_voltage  ,moni.drs_dvdd_voltage   );
      assert_eq!(result.drs_dvdd_current  ,moni.drs_dvdd_current   );
      assert_eq!(result.drs_dvdd_power    ,moni.drs_dvdd_power     );
      assert_eq!(result.p3v3_voltage      ,moni.p3v3_voltage       );
      assert_eq!(result.p3v3_current      ,moni.p3v3_current       );
      assert_eq!(result.p3v3_power        ,moni.p3v3_power         );
      assert_eq!(result.zynq_voltage      ,moni.zynq_voltage       );
      assert_eq!(result.zynq_current      ,moni.zynq_current       );
      assert_eq!(result.zynq_power        ,moni.zynq_power         );
      assert_eq!(result.p3v5_voltage      ,moni.p3v5_voltage       );
      assert_eq!(result.p3v5_current      ,moni.p3v5_current       );
      assert_eq!(result.p3v5_power        ,moni.p3v5_power         );
      assert_eq!(result.adc_dvdd_voltage  ,moni.adc_dvdd_voltage   );
      assert_eq!(result.adc_dvdd_current  ,moni.adc_dvdd_current   );
      assert_eq!(result.adc_dvdd_power    ,moni.adc_dvdd_power     );
      assert_eq!(result.adc_avdd_voltage  ,moni.adc_avdd_voltage   );
      assert_eq!(result.adc_avdd_current  ,moni.adc_avdd_current   );
      assert_eq!(result.adc_avdd_power    ,moni.adc_avdd_power     );
      assert_eq!(result.drs_avdd_voltage  ,moni.drs_avdd_voltage   );
      assert_eq!(result.drs_avdd_current  ,moni.drs_avdd_current   );
      assert_eq!(result.drs_avdd_power    ,moni.drs_avdd_power     );
      assert_eq!(result.n1v5_voltage      ,moni.n1v5_voltage       );
      assert_eq!(result.n1v5_current      ,moni.n1v5_current       );
      assert_eq!(result.n1v5_power        ,moni.n1v5_power         );
      assert_eq!(result, moni);
    }
  }

  #[test]
  fn serialization_circle_test_for_rbeventheader() {
    // try this 100 times
    for _n in 0..100 {
      let header       = RBEventHeader::from_random();
      let header_ser   = header.to_bytestream();
      let mut pos = 0usize;
      let header_deser = RBEventHeader::from_bytestream(&header_ser, &mut pos);
      assert_eq!(header_deser.unwrap(), header);
    }
  }

  #[test]
  fn extract_eventid_for_rbeventheader() {
    for _n in 0..100 {
      let header   = RBEventHeader::from_random();
      let event_id = RBEventHeader::extract_eventid_from_rbheader(&header.to_bytestream()); 
      assert_eq!(event_id, header.event_id);
    }
  }

  #[test]
  fn extract_rbheader_from_rbbinarydump() {
    for _n in 0..100 {
      let data   = RBBinaryDump::from_random();
      let stream = data.to_bytestream();
      let header = RBEventHeader::extract_from_rbbinarydump(&stream, &mut 0).unwrap();
      //assert_eq!(header.rb_id as u16, data.id);
      assert_eq!(header.channel_mask as u16, data.ch_mask);
      assert_eq!(header.event_id, data.event_id);
      //assert_eq!(header.stop_cell, data.stop_cell);
      //assert_eq!(header.crc32, data.crc32);
    }
  }

}

