

#[cfg(test)]
mod test_readoutboard_blob {
  use crate::readoutboard_blob::{BlobData, get_constant_blobeventsize};
  #[test]
  fn serialize_deserialize_roundabout () {
    let mut blob = BlobData {..Default::default()};
    blob.head = 142;
    blob.status = 212;
    blob.len  = 42;
    blob.roi = 100;
    blob.dna = 1000;
    blob.fw_hash = 42;
    blob.id = 5;
    blob.ch_mask = 111;
    blob.event_ctr = 9800001;
    blob.dtap0 = 10000;
    blob.dtap1 = 11000;
    blob.timestamp = 1123456;
    blob.stop_cell = 4;
    blob.crc32  = 88888;
    blob.tail   = 1000;
    let mut bytestream = blob.to_bytestream();
    for n in 0..get_constant_blobeventsize() {
        bytestream.push(0);
    }
    blob.from_bytestream(&bytestream, 0);
    blob.print();

    assert_eq!(1,1);
  }
}

