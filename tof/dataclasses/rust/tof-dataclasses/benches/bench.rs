use criterion::{
    //use tof_dataclasses::events::TofEvent;black_box, 
    criterion_group,
    criterion_main,
    Criterion
};

#[macro_use]
extern crate log;
use std::path::Path;

//use tempfile::NamedTempFile;

use tof_dataclasses::events::{
    RBEvent
};
//use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::io::{
    read_file,
    RBEventMemoryStreamer,
    TofPacketWriter, 
    TofPacketReader,
    FileType
};

use tof_dataclasses::events::TofEvent;
use tof_dataclasses::packets::TofPacket;

use tof_dataclasses::serialization::{
    Packable
};
use tof_dataclasses::FromRandom;

// setup tests

fn write_testfile(nevents : usize) -> String {
  let fname = String::from("/tmp/");
  //let fname_c = fname.clone();
  let mut writer = TofPacketWriter::new(fname.clone(), FileType::RunFile(1));
  for _ in 0..nevents {
    let tp = TofPacket::from_random();
    writer.add_tof_packet(&tp);
  }
  writer.file_name.clone()
}


fn tpreader_read() {
  let fname = write_testfile(1000);
  //let fname = data.path().to_str().map(|s| s.to_owned()).unwrap();
  let mut reader = TofPacketReader::new(fname);
  for tp in reader.next() {
    //foo
  }
}

// FIXME - remember to measure throuput as well
// group.throughput(Throughput::Bytes(bytes.len() as u64));

fn pack_tofevent_worst() {
  for _ in 0..100 {
    let mut te   = TofEvent::new();
    for _ in 0..40 {
      let mut rbev = RBEvent::new();
      for k in 0..8 {
        let data = vec![0u16;1024];
        rbev.adc[k] = data;
      }
      te.rb_events.push(rbev);
    }
    let pack = te.pack();
    let _ : TofEvent = pack.unpack().unwrap();
  }
}

fn pack_tofevent_best() {
  for _ in 0..100 {
    let mut te   = TofEvent::new();
    for _ in 0..1 {
      let mut rbev = RBEvent::new();
      for k in 0..1 {
        let data = vec![0u16;1024];
        rbev.adc[k] = data;
      }
      te.rb_events.push(rbev);
    }
    let pack = te.pack();
    let _ : TofEvent = pack.unpack().unwrap();
  }
}

fn pack_tofevent_average() {
  for _ in 0..100 {
    let mut te   = TofEvent::new();
    for _ in 0..4 {
      let mut rbev = RBEvent::new();
      for k in 0..2 {
        let data = vec![0u16;1024];
        rbev.adc[k] = data;
      }
      te.rb_events.push(rbev);
    }
    let pack = te.pack();
    let _ : TofEvent = pack.unpack().unwrap();
  }
}

fn bench_pack_tofevent_worst(c: &mut Criterion) {
  c.bench_function("pack_tofevent_worst", |b|
                   b.iter(|| pack_tofevent_worst()));
}

fn bench_pack_tofevent_best(c: &mut Criterion) {
  c.bench_function("pack_tofevent_best", |b|
                   b.iter(|| pack_tofevent_best()));
}

fn bench_pack_tofevent_average(c: &mut Criterion) {
  c.bench_function("pack_tofevent_average", |b|
                   b.iter(|| pack_tofevent_average()));
}

fn pack_rbevent() {
  let ev = RBEvent::from_random();
  let _ : RBEvent = ev.pack().unpack().unwrap();
}

fn bench_tpreader_read(c: &mut Criterion) {
  c.bench_function("tpreader_read", |b| 
                   b.iter(|| tpreader_read()));
}


fn bench_pack_rbevent(c: &mut Criterion) {
  c.bench_function("pack_rbevent", |b|
                   b.iter(|| pack_rbevent()));
}

fn bench_read_file(c: &mut Criterion) {
  c.bench_function("read_file", |b|
                   b.iter(|| read_file(&Path::new("test-data/tof-rb01.robin")).unwrap()));
}

fn bench_streamer_index_helper() {
  let mut streamer = RBEventMemoryStreamer::new();
  let mut data = read_file(&Path::new("test-data/tof-rb01.robin")).unwrap();
  streamer.consume(&mut data);      
  streamer.create_event_index();
}

fn bench_streamer_next_helper() {
  let mut streamer = RBEventMemoryStreamer::new();
  let mut data = read_file(&Path::new("test-data/tof-rb01.robin")).unwrap();
  streamer.consume(&mut data);      
      //RobinReader::new(("test-data/tof-rb01.robin").to_string());
  //reader.cache_all_events();
  let mut nevents = 0usize;
  streamer.create_event_index();
  //streamer.print_event_map();
  loop {
    match streamer.next() {
      None => break,
      Some(_) => {
        nevents += 1;
        continue;
      }
    }
  }
  debug!("extracted {} events!", nevents);
}

fn bench_streamer_tofpacket_helper() {
  let mut streamer = RBEventMemoryStreamer::new();
  let mut data = read_file(&Path::new("test-data/tof-rb01.robin")).unwrap();
  streamer.consume(&mut data);      
      //RobinReader::new(("test-data/tof-rb01.robin").to_string());
  //reader.cache_all_events();
  let mut nevents = 0usize;
  streamer.create_event_index();
  //streamer.print_event_map();
  loop {
    match streamer.next_tofpacket() {
      None => break,
      Some(_) => {
        nevents += 1;
        continue;
      }
    }
  }
  debug!("extracted {} events!", nevents);
}

fn bench_streamer_index(c: &mut Criterion) {
  c.bench_function("streamer_index", |b|
                   b.iter(|| bench_streamer_index_helper()));
}

fn bench_streamer_next(c: &mut Criterion) {
  c.bench_function("streamer_next", |b|
                   b.iter(|| bench_streamer_next_helper()));
}

fn bench_streamer_tofpacket(c: &mut Criterion) {
  c.bench_function("streamer_tofpacket", |b|
                   b.iter(|| bench_streamer_tofpacket_helper()));
}




criterion_group!(name = benches;
                 config = Criterion::default().sample_size(10);
                 targets = bench_read_file,
                           bench_streamer_tofpacket,
                           bench_streamer_index,
                           bench_streamer_next,
                           bench_tpreader_read);

cfg_if::cfg_if! {
  if #[cfg(feature = "random")] {
    criterion_group!(benches_random,
                     bench_pack_tofevent_worst,
                     bench_pack_tofevent_best,
                     bench_pack_tofevent_average,
                     bench_pack_rbevent,
                     );
    criterion_main!(benches, benches_random);
  } else {
    criterion_main!(benches);
  }
}



