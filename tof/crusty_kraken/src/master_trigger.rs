/****
 *
 * Communications with the 
 * mastertrigger
 *
 */ 


use std::net::UdpSocket;

pub const MT_MAX_PACKSIZE   : usize = 4096;


#[derive(PartialEq, Copy, Clone)]
enum PACKET_TYPE {
  READ           = 0,
  WRITE          = 1,
  READ_NON_INCR  = 2,
  WRITE_NON_INCR = 3,
  RMW            = 4
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

fn read_event_cnt(socket : &UdpSocket,
                  target_address : &str,
                  buffer : &mut [u8;MT_MAX_PACKSIZE]) {
  let event_count = read_register(socket, target_address, 0xd, buffer);
  println!("Got event count! {} ", event_count);
}


///
/// Communications with the master trigger
///
///
pub fn master_and_commander(mt_ip   : &str, 
                            mt_port : usize) {

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
  
  // this is not strrictly necessary, but 
  // it is nice to limit communications
  match socket.connect(&mt_address) {
    Err(err) => panic!("Can not connect to master trigger at {}, err {}", mt_address, err),
    Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
  }
  
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  loop {
  //  let received = socket.recv_from(&mut buffer);

  //  match received {
  //    Ok((size, addr)) => println!("Received {} bytes from address {}", size, addr),
  //    Err(err)         => {
  //      println!("Received nothing! err {}", err);
  //      continue;
  //    }
  //  } // end match
    read_event_cnt(&socket, &mt_address, &mut buffer);
  } // end loop
}

