use tof_dataclasses::io::RBEventMemoryStreamer;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use log::error;
extern crate pretty_env_logger;


fn main() {
  pretty_env_logger::init();
  let filename = String::from("/data0/gaps/nts/raw/1/RB12_1686442529.blob");
  let path = Path::new(&filename); 
  let file = OpenOptions::new().create(false).append(false).read(true).open(path).expect("Unable to open file {filename}");
  let mut file_reader = BufReader::new(file);
  let mut read_bytes  = 1usize;
  const CHUNKSIZE : usize  = 200000;
  //let mut buffer      = [0u8;CHUNKSIZE];
  let mut total_bytes_read = 0usize;
  let mut streamer    = RBEventMemoryStreamer::new();
  let mut n_events    = 0usize;
  let mut event_ids   = Vec::<u32>::new();
  while read_bytes != 0 {
    let mut buffer      = [0u8;CHUNKSIZE];
    match file_reader.read(&mut buffer) {
      Err(err) => {
        error!("Unable to read any bytes from file {}, {}", filename, err);
      },
      Ok(_nbytes) => {
        if _nbytes == 0 {
          break
        }
        read_bytes = _nbytes;
        total_bytes_read += read_bytes;
        //for k in 3970..3980 {
        //  println!("--> read bytes {}", buffer[k]);
        //}
        //let headfound = search_for_u16(0xaaaa, &buffer.to_vec(), 0); 
        //let tailfound = search_for_u16(0x5555, &buffer.to_vec(), 0); 
        //let headfound2 = search_for_u16(0xaaaa, &buffer.to_vec(), tailfound.unwrap_or(0)); 
        //println!("Head found at {}", headfound.unwrap_or(99999));
        //println!("Tail found at {}", tailfound.unwrap_or(99999));
        //println!("Head2 found at {}", headfound2.unwrap_or(99999));
        streamer.add(&buffer.to_vec(), _nbytes);
        //println!("Read {} bytes, stream len {}", _nbytes, streamer.stream.len());
        while streamer.stream.len() > 44 {
          match streamer.next() {
            None => {
              println!("none..");
              break;
            },
            Some(event) => {
              println!("{}", event);
              if event.header.rb_id != 12 {
                println!("{}", event);
                //if !event.header.broken {
                // panic!("Wrong rb id");
                //}
              }
              n_events += 1;
              event_ids.push(event.header.event_id);
              println!("==> Nevents {}", n_events);
              //if n_events == 14 {
              //  panic!("pan");
              //}
            }
          }

          //println!("{}", event);
        }
      }
    }
  }
  println!("Get  {} event ids!", event_ids.len());
  event_ids.dedup();
  println!("After removing duplicates, we got {} event ids!", event_ids.len());
  println!("Read {} bytes from {}", total_bytes_read, filename);
}
