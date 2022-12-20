/****
 *
 * Communications with the 
 * mastertrigger
 *
 */ 

// to measure the rate
use std::time::{Duration, Instant};
use std::thread;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;

//const MT_MAX_PACKSIZE   : usize = 4096;
const MT_MAX_PACKSIZE   : usize = 512;


#[derive(PartialEq, Copy, Clone)]
enum PACKET_TYPE {
  READ           = 0,
  WRITE          = 1,
  READ_NON_INCR  = 2,
  WRITE_NON_INCR = 3,
  RMW            = 4
}


fn count_ones(input :u32) -> u32 {
  let mut count = 0u32;
  let mut value = input;
  while value > 0 {
    count += value & 1;
    value >>= 1;
  }
  count
}


///
///
///
///
fn encode_ipbus(addr        : u32,
                packet_type : PACKET_TYPE,
                data        : &Vec<u32>) -> Vec<u8> {
  // this will silently overflow, but 
  // if the message is that long, then 
  // most likely there will be a 
  // problem elsewhere, so we 
  // don't care
  let size = data.len() as u8;

  let PACKET_ID = 0u8;
  let mut udp_data = Vec::<u8>::from([
    // Transaction Header
    0x20 as u8, // Protocol version & RSVD
    0x00 as u8, // Transaction ID (0 or bug)
    0x00 as u8, // Transaction ID (0 or bug)
    0xf0 as u8, // Packet order & packet_type
    // Packet Header
    //
    // FIXME - in the original python script, 
    // the 0xf0 is a 0xf00, but this does not
    // make any sense in my eyes...
    (0x20 as u8 | ((PACKET_ID & 0xf0 as u8) as u32 >> 8) as u8), // Protocol version & Packet ID MSB
    (PACKET_ID & 0xff as u8), // Packet ID LSB,
    size, // Words
    (((packet_type as u8 & 0xf as u8) << 4) | 0xf as u8), // Packet_Type & Info code
    // Address
    ((addr & 0xff000000 as u32) >> 24) as u8,

    ((addr & 0x00ff0000 as u32) >> 16) as u8,
    ((addr & 0x0000ff00 as u32) >> 8)  as u8,
    (addr  & 0x000000ff as u32) as u8]);

  if packet_type == PACKET_TYPE::WRITE
     || packet_type == PACKET_TYPE::WRITE_NON_INCR {
    for i in 0..size as usize {
      udp_data.push (((data[i] & 0xff000000 as u32) >> 24) as u8);
      udp_data.push (((data[i] & 0x00ff0000 as u32) >> 16) as u8);
      udp_data.push (((data[i] & 0x0000ff00 as u32) >> 8)  as u8);
      udp_data.push ( (data[i] & 0x000000ff as u32)        as u8);
    }
  }
  //for n in 0..udp_data.len() {
  //    println!("-- -- {}",udp_data[n]);
  //}
  udp_data
}

///
///
///
///
///
///
fn decode_ipbus( message : &[u8;MT_MAX_PACKSIZE],
                 verbose : bool) -> Vec<u32> {

    // Response
    let ipbus_version = message[0] >> 4;
    let id            = (((message[4] & 0xf as u8) as u32) << 8) as u8 | message[5];
    let size          = message[6];
    let packet_type   = (message[7] & 0xf0 as u8) >> 4;
    let info_code     = message[7] & 0xf as u8;
    let mut data      = Vec::<u32>::new(); //[None]*size

    // Read

    if matches!(PACKET_TYPE::READ, packet_type)
    || matches!(PACKET_TYPE::READ_NON_INCR, packet_type) {
      for i in 0..size as usize {
        data.push(  ((message[8 + i * 4]  as u32) << 24) 
                  | ((message[9 + i * 4]  as u32) << 16) 
                  | ((message[10 + i * 4] as u32) << 8)  
                  |  message[11 + i * 4]  as u32)
      }
    }

    // Write
    if matches!(PACKET_TYPE::WRITE, packet_type) {
        data.push(0);
    }
    if verbose { 
      println!("Decoding IPBus Packet:");
      println!(" > Msg = {:?}", message);
      println!(" > IPBus version = {}", ipbus_version);
      println!(" > ID = {}", id);
      println!(" > Size = {}", size);
      println!(" > Type = {}", packet_type);
      println!(" > Info = {}", info_code);
      println!(" > data = {:?}", data);
    }
    data
}

///
///
///
///
fn read_register(socket      : &UdpSocket,
                 target_addr : &str,
                 reg_addr    : u32,
                 buffer      : &mut [u8;MT_MAX_PACKSIZE]) -> u32 {
  let send_data = Vec::<u32>::from([0]);
  let message   = encode_ipbus(reg_addr,
                               PACKET_TYPE::READ,
                               &send_data);
  socket.send_to(message.as_slice(), target_addr);
  let (number_of_bytes, src_addr) = socket.recv_from(buffer).expect("No data!");
  trace!("Received {} bytes from master trigger", number_of_bytes);
  let data = decode_ipbus(buffer, false)[0];
  data
}

// FIXME - there is no verification step!
fn write_register(socket      : &UdpSocket,
                  target_addr : &str,
                  reg_addr    : u32,
                  data        : u32,
                  buffer      : &mut [u8;MT_MAX_PACKSIZE]){

  let send_data = Vec::<u32>::from([data]);
  let message   = encode_ipbus(reg_addr,
                               PACKET_TYPE::WRITE,
                               &send_data);
  socket.send_to(message.as_slice(), target_addr);
  let (number_of_bytes, src_addr) = socket.recv_from(buffer).expect("No data!");
  trace!("Received {} bytes from master trigger", number_of_bytes);
  //let data = decode_ipbus(buffer, false)[0];
//def wReg(address, data, verify=False):
//    s.sendto(encode_ipbus(addr=address, packet_type=WRITE, data=[data]), target_ad    dress)
//    s.recvfrom(4096)
//    rdback = rReg(address)
//    if (verify and rdback != data):
//        print("Error!")
//
}

fn read_event_cnt(socket : &UdpSocket,
                  target_address : &str,
                  buffer : &mut [u8;MT_MAX_PACKSIZE]) -> u32 {
  let event_count = read_register(socket, target_address, 0xd, buffer);
  trace!("Got event count! {} ", event_count);
  event_count
}

fn reset_event_cnt(socket : &UdpSocket,
                   target_address : &str) {
  debug!("Resetting event counter!");
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket, target_address, 0xc,1,&mut buffer);
}

fn reset_daq(socket : &UdpSocket,
             target_address : &str) {
  debug!("Resetting DAQ!");
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket, target_address, 0x10, 1,&mut buffer);
}

fn read_daq(socket : &UdpSocket,
            target_address : &str,
            buffer : &mut [u8;MT_MAX_PACKSIZE]) -> (u32, u32) {
  // check if the queue is full
  let mut event_ctr  = 0u32;
  let mut timestamp  = 0u32;
  let mut timecode32 = 0u32;
  let mut timecode16 = 0u32;
  let mut mask       = 0u32;
  //let mut hits       = 0u32;
  let mut crc        = 0u32;
  let mut trailer    = 0u32;

  let word = read_register(socket, target_address, 0x11, buffer);
  let mut hit_paddles = 0u32;
  // this will eventually determin, 
  // how often we will read the 
  // hit register
  let mut paddles_rxd     = 1u32;
  let mut hits = Vec::<u32>::with_capacity(24);
  if word == 0xAAAAAAAA {
    // we start a new daq package
    event_ctr   = read_register(socket, target_address, 0x11, buffer);
    timestamp   = read_register(socket, target_address, 0x11, buffer);
    timecode32  = read_register(socket, target_address, 0x11, buffer);
    timecode16  = read_register(socket, target_address, 0x11, buffer);
    mask        = read_register(socket, target_address, 0x11, buffer);
    hit_paddles = count_ones(mask);
    hits.push     (read_register(socket, target_address, 0x11, buffer));
    //allhits.push(hits);  
    while paddles_rxd < hit_paddles {
      hits.push(read_register(socket, target_address, 0x11, buffer));
      paddles_rxd += 1;
    }
    crc         = read_register(socket, target_address, 0x11, buffer);
    trailer     = read_register(socket, target_address, 0x11, buffer);

    debug!("event_ctr {}, ts {} , tc32 {}, tc16 {}, mask {}, crc {}, trailer {}", event_ctr, timestamp, timecode32, timecode16, hit_paddles, crc, trailer);
    for n in 0..hits.len() {
      debug!(" -- -- hit {}", hits[n]);
    }
  } // end header found
    //AAAAAAAA (Header)
    //0286A387 (Event cnt)
    //00000000 (Timestamp)
    //00000000 (Timecode 32 bits)
    //00000000 (Timecode 16 bits)
    //00000001 (Mask)
    //00000003 (Hits)
    //97041A48 (CRC)
    //55555555 (Trailer)
  (event_ctr, hit_paddles)
}

///
/// Communications with the master trigger
///
///
pub fn master_and_commander(mt_ip   : &str, 
                            mt_port : usize,
                            evid_sender : &Sender<(u32, u32)>) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
  //let mut socket : UdpSocket;
  // FIXME - proper error checking
  let local_port = "0.0.0.0:50100";
  let local_socket = UdpSocket::bind(local_port);
  let mut socket : UdpSocket;
  match local_socket {
    Err(err)   => panic!("Can not create local UDP port for master trigger connection at {}!, err {}", local_port, err),
    Ok(value)  => {
      info!("Successfully bound UDP socket for master trigger communcations to {}", local_port);
      socket = value;
    }
  } // end match
 
  //socket.set_nonblocking(true).unwrap();
  
  
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  let mut event_cnt      = 0u32;
  let mut last_event_cnt = 0u32;
  let mut missing_evids  = 0usize;
  let mut event_missing  = false;
  let mut n_events       = 0usize;
  // these are the number of expected events
  // (missing included)
  let mut n_events_expected = 0usize;
  let mut n_paddles_expected : u32;
  let mut rate = 0f64;
  // for rate measurement
  let start = Instant::now();

  // limit polling rate to a maximum
  let max_rate = 1000.0; // hz
    
  // this is not strrictly necessary, but 
  // it is nice to limit communications
  match socket.connect(&mt_address) {
    Err(err) => panic!("Can not connect to master trigger at {}, err {}", mt_address, err),
    Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
  }
  // reset the master trigger before acquisiton
  reset_daq(&socket, &mt_address);  
  reset_event_cnt(&socket, &mt_address); 
 
  loop {
    // limit the max polling rate
    let milli_sleep = Duration::from_millis((1000.0/max_rate) as u64);
    thread::sleep(milli_sleep);
  
    //match socket.connect(&mt_address) {
    //  Err(err) => panic!("Can not connect to master trigger at {}, err {}", mt_address, err),
    //  Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
    //}
    //  let received = socket.recv_from(&mut buffer);

    //  match received {
    //    Ok((size, addr)) => println!("Received {} bytes from address {}", size, addr),
    //    Err(err)         => {
    //      println!("Received nothing! err {}", err);
    //      continue;
    //    }
    //  } // end match
    
    // daq queue states
    // 0 - full
    // 1 - something
    // 2 - empty
    //if 0 != (read_register(&socket, &mt_address, 0x12, &mut buffer) & 0x2) {
    if read_register(&socket, &mt_address, 0x12, &mut buffer) == 2 {
      info!("No new information from DAQ");
      //reset_daq(&socket, &mt_address);  
      continue;
    }
    
    //event_cnt = read_event_cnt(&socket, &mt_address, &mut buffer);
    (event_cnt, n_paddles_expected) = read_daq(&socket, &mt_address, &mut buffer);
    if event_cnt == last_event_cnt {
      info!("Same event!");
      continue;
    }

    // we have a new event
    //println!("** ** evid: {}",event_cnt);
    if event_cnt < last_event_cnt {
      error!("Event counter id overflow!!");
      last_event_cnt = 0;
    }
    
    if event_cnt - last_event_cnt > 1 {
      let missing = event_cnt - last_event_cnt;
      
      // FIXME
      if missing < 200 {
        missing_evids += missing as usize;
      } else {
        warn!("We missed too many event ids from the master trigger!");
      }
      //error!("We missed {} events!", missing);
      event_missing = true;
    }
    
    // new event
    // send it down the pip
    evid_sender.send((event_cnt, n_paddles_expected));
    last_event_cnt = event_cnt;
    n_events += 1;
    n_events_expected = n_events + missing_evids;

    let elapsed = start.elapsed().as_secs();
    // measure rate every 100 events
    if n_events % 10 == 0 {
      rate = n_events as f64 / elapsed as f64;
      println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      rate = n_events_expected as f64 / elapsed as f64;
      println!("==> -- expected rate {:.3} Hz", rate);   
    } 
    // end new event

    // a heartbeat every 10 s
    let elapsed = start.elapsed().as_secs();
    if elapsed % 10 == 0 {
      println!("== == == == == == == == HEARTBEAT! {} seconds passed!", elapsed);
      rate = n_events as f64 / elapsed as f64;
      println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      rate = n_events_expected as f64 / elapsed as f64;
      println!("==> -- expected rate {:.3} Hz", rate);   
      println!("== == == == == == == == END HEARTBEAT!");
    }

  } // end loop
}

