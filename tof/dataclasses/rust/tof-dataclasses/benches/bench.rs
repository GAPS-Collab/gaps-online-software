use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[macro_use]
extern crate log;
use std::path::Path;
use tof_dataclasses::events::{
    RBEventMemoryView,
    RBEvent
};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::io::{
    read_file,
    RobinReader,
    RBEventMemoryStreamer
};

use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::FromRandom;


// FIXME - remember to measure throuput as well
// group.throughput(Throughput::Bytes(bytes.len() as u64));

fn bench_tofpacket_serialize_helper() {
  let mut n_iter = 0u32;
  let ev = RBEvent::from_random();
  loop {
    let tp = TofPacket::from(&ev);
    let stream = tp.to_bytestream();
    match TofPacket::from_bytestream(&stream, &mut 0) {
      Err(err) => {
        error!("Can't unpack stream! {err}");
      },
      Ok(_) => (),
    }
    n_iter += 1;
    if n_iter == 100 {
      break;
    }
  }
}


fn bench_multitofpacket_serialize_helper() {
  let mut n_iter = 0u32;
  let ev = RBEvent::from_random();
  //let mut packets = Vec::<TofPacket>::with_capacity(1000);
  let mut packets = Vec::<TofPacket>::new();
  loop {
    let tp = TofPacket::from(&ev);
    packets.push(tp);
    //let stream = tp.to_bytestream();
    //match TofPacket::from_bytestream(&stream, &mut 0) {
    //  Err(err) => {
    //    error!("Can't unpack stream!");
    //  },
    //  Ok(_) => (),
    //}
    n_iter += 1;
    if n_iter == 100 {
      break;
    }
  }
}

fn bench_multitofpacket_serialize(c: &mut Criterion) {
  c.bench_function("multitofpacket_serialize", |b|
                   b.iter(|| bench_multitofpacket_serialize_helper()));
}

fn bench_tofpacket_serialize(c: &mut Criterion) {
  c.bench_function("tofpacket_serialize", |b|
                   b.iter(|| bench_tofpacket_serialize_helper()));
}

fn bench_read_file(c: &mut Criterion) {
  c.bench_function("read_file", |b|
                   b.iter(|| read_file(&Path::new("test-data/tof-rb01.robin")).unwrap()));
}

fn bench_rreader_read_helper() {
  let mut reader = RobinReader::new(("test-data/tof-rb01.robin").to_string());
  //reader.cache_all_events();
  let mut nevents = 0usize;
  loop {
    match reader.next() {
      None => {
        break;
      },
      Some(ev) => {
        nevents += 1;
        let _ = TofPacket::from(&ev);
        continue;
      }
    }
  }
  //println!("extracted {} events!", nevents);
}

fn bench_rbeventmemoryview_readfile_helper() {
  let data = read_file(&Path::new("test-data/tof-rb01.robin")).unwrap();
  let mut pos = 0usize;
  let mut nevents = 0usize;
  while data.len() - pos > 18530 {
    let _ = RBEventMemoryView::from_bytestream(&data, &mut pos);
    nevents += 1;
  }
  //println!("extracted {} events", nevents);
}

fn bench_rbeventmemoryview_readfile(c: &mut Criterion) {
  c.bench_function("rreventmemoryview_readfile", |b|
                   b.iter(|| bench_rbeventmemoryview_readfile_helper()));
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

fn bench_rbevent_serialization_helper() {
  let ev = RBEvent::from_random();
  let result = RBEvent::from_bytestream(&ev.to_bytestream(), &mut 0).unwrap();
  let _  = TofPacket::from(&result);
}

fn bench_rbevent_serialization(c: &mut Criterion) {
  c.bench_function("rbevent_serialization", |b|
                   b.iter(|| bench_rbevent_serialization_helper()));
}


fn bench_rreader_read(c: &mut Criterion) {
  c.bench_function("rreader_read", |b|
                   b.iter(|| bench_rreader_read_helper()));
}

cfg_if::cfg_if! {
  if #[cfg(feature = "random")] {
    fn bench_rbmemoryview_from_random(c: &mut Criterion) {
      c.bench_function("rbmemoryview_fromrandom", |b|
                       b.iter(|| RBEventMemoryView::from_random()));
    }


    fn bench_rbmemoryview_serialization_circle_helper() {
      let data   = RBEventMemoryView::from_random();
      let stream = data.to_bytestream();
      let _      = RBEventMemoryView::from_bytestream(&stream, &mut 0);
    }

    fn bench_rbmemoryview_serialization_circle(c: &mut Criterion) {
      c.bench_function("rbinbary_dump_serialization_circle", |b|
        b.iter(|| bench_rbmemoryview_serialization_circle_helper()) 
      );
    }

    fn bench_rbmemoryview_serialization_circle_norandom(c: &mut Criterion) {
      c.bench_function("rbinbary_dump_serialization_circle_norandom", |b|
        b.iter(|| {
          let data = RBEventMemoryView::new();
          let _ = RBEventMemoryView::from_bytestream(&data.to_bytestream(), &mut 0);
        }) 
      );
    }
  }
}

criterion_group!(name = benches;
                 config = Criterion::default().sample_size(10);
                 targets = bench_read_file, bench_rreader_read, bench_rbeventmemoryview_readfile, bench_streamer_tofpacket, bench_streamer_index, bench_streamer_next,
                 );

cfg_if::cfg_if! {
  if #[cfg(feature = "random")] {
    criterion_group!(benches_random,
                     bench_multitofpacket_serialize,
                     bench_tofpacket_serialize,
                     bench_rbmemoryview_from_random,
                     bench_rbmemoryview_serialization_circle,
                     bench_rbmemoryview_serialization_circle_norandom,
                     bench_rbevent_serialization
                     );
    criterion_main!(benches, benches_random);
  } else {
    criterion_main!(benches);
  }
}



