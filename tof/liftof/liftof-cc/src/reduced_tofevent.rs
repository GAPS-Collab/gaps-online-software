///! Event strucutures for data reconrded byi the tof
///
///  These are used internally, and will get
///  serialized and send over the write
///
///
///
///

use std::time::SystemTime;

use crate::constants::EVENT_TIMEOUT;
//use crate::errors::SerializationError;
use crate::errors::EventError;

use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::serialization::search_for_u16;
use tof_dataclasses::errors::SerializationError;

#[cfg(feature="random")]
use rand::Rng;


///! Microseconds since epock
fn elapsed_since_epoch() -> u128 {
  SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros()
}

//
/////! check for a certain number in a bytestream
//fn search_for_u16(number : u16, bytestream : &Vec<u8>, start_pos : usize) 
//  -> Result<usize, SerializationError> {
//
//  if start_pos > bytestream.len() {
//    return Err(SerializationError::StreamTooShort);
//  }
//
//  let mut pos = start_pos;
//
//  let mut two_bytes : [u8;2]; 
//  // will find the next header
//  two_bytes = [bytestream[pos], bytestream[pos + 1]];
//  pos += 2;
//  if (u16::from_le_bytes(two_bytes)) != PaddlePacket::Head {
//    // we search for the next packet
//    for n in pos..bytestream.len() {
//      two_bytes = [bytestream[pos], bytestream[pos + 1]];
//      if (u16::from_le_bytes(two_bytes)) != number {
//        pos += n;
//        break;
//      }
//    }
//    return Err(SerializationError::ValueNotFound);
//  }
//  Ok(pos)
//}
//
//
/////! Representation of analyzed data from a paddle
/////
/////  Holds the results of waveform analysis for a 
/////  paddle, that is the reustl for 2 individual 
/////  waveforms from each end of the paddle.
/////
/////  Thus it is having methods like `get_time_a`
/////  where a and be refere to different 
/////  paddle ends.
/////
/////
//#[derive(Debug,Copy,Clone, PartialEq)]
//pub struct PaddlePacket  {
//  
//  //unsigned short head = 0xF0F0;
//  pub paddle_id    : u8,
//  pub time_a       : u16,
//  pub time_b       : u16,
//  pub peak_a       : u16,
//  pub peak_b       : u16,
//  pub charge_a     : u16,
//  pub charge_b     : u16,
//  pub charge_min_i : u16,
//  pub pos_across   : u16,
//  pub t_average    : u16,
//  pub ctr_etx      : u8,
//
//  // fields which won't get 
//  // serialized
//  pub event_id     : u32,
//  pub valid        : bool
//}
//
//impl PaddlePacket {
//
//  pub const PacketSize    : usize = 24;
//  pub const Version       : &'static str = "1.1";
//  pub const Head          : u16  = 61680; //0xF0F0)
//  pub const Tail          : u16  = 3855;
//
//  pub fn new() -> PaddlePacket {
//    PaddlePacket{
//                  paddle_id    : 0,
//                  time_a       : 0,
//                  time_b       : 0,
//                  peak_a       : 0,
//                  peak_b       : 0,
//                  charge_a     : 0,
//                  charge_b     : 0,
//                  charge_min_i : 0,
//                  pos_across   : 0,
//                  t_average    : 0,
//                  ctr_etx      : 0,
//                  // non-serialize fields
//                  event_id     : 0,
//                  valid        : true
//                }
//
//  }
//
//  pub fn invalidate(&mut self) {
//    self.valid = false;
//  }
//
//  pub fn set_time_a(&mut self, time : f64 ) {
//    let prec : f64 = 0.004;
//    self.time_a = (time as f64/prec) as u16;
//  }
//
//  pub fn set_time_b(&mut self, time : f64 ) {
//    let prec : f64 = 0.004;
//    self.time_b = (time as f64/prec) as u16;
//  }
//  
//  pub fn set_time(&mut self, time : f64, side : usize ) {
//    assert!(side == 0 || side == 1);
//    if side == 0 {self.set_time_a(time);}
//    if side == 1 {self.set_time_b(time);}
//  }
//
//  pub fn reset(&mut self) {
//    self.paddle_id    =  0;
//    self.time_a       =  0;
//    self.time_b       =  0;
//    self.peak_a       =  0;
//    self.peak_b       =  0;
//    self.charge_a     =  0;
//    self.charge_b     =  0;
//    self.charge_min_i =  0;
//    self.pos_across   =  0;
//    self.t_average    =  0;
//    self.ctr_etx      =  0;
//    self.event_id     =  0;
//    self.valid        =  true;
//  }
//
//
//  pub fn print(&self)
//  {
//    println!("***** paddle packet *****");
//    println!("==> VALID        {}", self.valid);
//    println!("=> time_a        {}", self.time_a);
//    println!("=> time_b        {}", self.time_b);
//    println!("=> peak_a        {}", self.peak_a);
//    println!("=> peak_b        {}", self.peak_b);
//    println!("=> charge_a      {}", self.charge_a);
//    println!("=> charge_b      {}", self.charge_b);
//    println!("=> charge_min_i  {}", self.charge_min_i);
//    println!("=> pos_across    {}", self.pos_across);
//    println!("=> t_average     {}", self.t_average);
//    println!("=> ctr_etx       {}", self.ctr_etx);
//    println!("*****");
//  }
//
//  ///! Serialize the packet
//  ///
//  ///  Not all fields witll get serialized, 
//  ///  only the relevant data for the 
//  ///  flight computer
//  ///
//  pub fn to_bytestream(&self) -> Vec<u8> {
//
//    let mut bytestream = Vec::<u8>::with_capacity(PaddlePacket::PacketSize);
//
//    bytestream.extend_from_slice(&PaddlePacket::Head.to_le_bytes());
//    bytestream.push(self.paddle_id); 
//    bytestream.extend_from_slice(&self.time_a      .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.time_b      .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.peak_a      .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.peak_b      .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.charge_a    .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.charge_b    .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.charge_min_i.to_le_bytes()); 
//    bytestream.extend_from_slice(&self.pos_across  .to_le_bytes()); 
//    bytestream.extend_from_slice(&self.t_average   .to_le_bytes()); 
//    bytestream.push(self.ctr_etx); 
//    bytestream.extend_from_slice(&PaddlePacket::Tail        .to_le_bytes()); 
//
//    bytestream
//  }
//
//
//  ///! Deserialization
//  ///
//  ///
//  ///  # Arguments:
//  ///
//  ///  * bytestream : 
//  pub fn from_bytestream(bytestream : &Vec<u8>, start_pos : usize) 
//    -> Result<(PaddlePacket),SerializationError> {
//    let mut pp  = PaddlePacket::new();
//    let mut pos = start_pos;
//    let mut two_bytes : [u8;2];
//
//    pos = search_for_u16(PaddlePacket::Head, &bytestream, pos)?;
//
//    pp.paddle_id = bytestream[pos];
//    pos += 1;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.time_a       =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.time_b       =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//    
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.peak_a       =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//    
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.peak_b       =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.charge_a     =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.charge_b     =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.charge_min_i =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.pos_across   =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    pp.t_average    =  u16::from_le_bytes(two_bytes);
//    pos += 2;
//
//    pp.ctr_etx      =  bytestream[pos];
//    pos += 1;
//
//    // at this postiion, there must be the footer
//    two_bytes = [bytestream[pos], bytestream[pos + 1]];
//    if (u16::from_le_bytes(two_bytes)) != PaddlePacket::Tail {
//      pp.valid = false;
//      return Err(SerializationError::TailInvalid);
//    }
//    pos += 2;
//    assert! ((pos - start_pos) == PaddlePacket::PacketSize);
//    pp.valid        =  true;
//    Ok(pp)
//  }
//
//  #[cfg(feature="random")]
//  pub fn from_random() -> PaddlePacket {
//    let mut pp = PaddlePacket::new();
//    let mut rng = rand::thread_rng();
//
//    pp.paddle_id    = rng.gen::<u8> ();
//    pp.time_a       = rng.gen::<u16>();
//    pp.time_b       = rng.gen::<u16>();
//    pp.peak_a       = rng.gen::<u16>();
//    pp.peak_b       = rng.gen::<u16>();
//    pp.charge_a     = rng.gen::<u16>();
//    pp.charge_b     = rng.gen::<u16>();
//    pp.charge_min_i = rng.gen::<u16>();
//    pp.pos_across   = rng.gen::<u16>();
//    pp.t_average    = rng.gen::<u16>();
//    pp.ctr_etx      = rng.gen::<u8>();
//
//    pp
//  }
//
//}
//

#[derive(Debug, Clone, PartialEq)]
pub struct TofEvent  {
  
  pub event_id     : u32,

  // this field can be debated
  // the reason we have it is 
  // that for de/serialization, 
  // we need to know the length 
  // of the expected bytestream.
  pub n_paddles    : u8, // we don't have more than 
                         // 256 paddles.
                         // HOWEVER!! For future gaps
                         // flights, we might...
                         // This will then overflow 
                         // and cause problems.
  
  // this is private, paddles can only 
  // be added
  pub paddle_packets : Vec::<PaddlePacket>,

  // fields which won't get 
  // serialized
  pub n_paddles_expected : u8,

  /// for the event builder. 
  /// if not using the master trigger,
  /// we can look at the time the event has first
  /// been seen and then it will be declared complete
  /// after timeout microseconds
  /// thus we are saving the time, this isntance has 
  /// been created.
  pub creation_time      : u128,

  pub valid              : bool,
}


impl TofEvent {
  
  pub const PacketSizeFixed    : usize = 24;
  pub const Version            : &'static str = "1.0";
  pub const Head               : u16  = 43690; //0xAAAA
  pub const Tail               : u16  = 21845; //0x5555
  

  pub fn new(event_id : u32,
             n_paddles_expected : u8) -> TofEvent {
    let creation_time  = SystemTime::now()
                         .duration_since(SystemTime::UNIX_EPOCH)
                         .unwrap().as_micros();

    TofEvent { 
      event_id       : event_id,
      n_paddles      : 0,  
      paddle_packets : Vec::<PaddlePacket>::with_capacity(20),

      n_paddles_expected : n_paddles_expected,

      // This is strictly for when working
      // with event timeouts
      creation_time  : creation_time,

      valid          : true,
    }
  }

  pub fn from_bytestream(bytestream : &Vec<u8>, start_pos : usize)
     -> Result<TofEvent, SerializationError> {
    let mut event = TofEvent::new(9,0);
    let mut pos = start_pos;

    pos = search_for_u16(TofEvent::Head, &bytestream, pos)?;
   
    let mut raw_bytes_4  = [bytestream[pos + 1],
                            bytestream[pos + 0],
                            bytestream[pos + 3],
                            bytestream[pos + 2]];
    pos   += 4; 
    event.event_id = u32::from_be_bytes(raw_bytes_4); 

    event.n_paddles      = bytestream[pos];
    pos += 1; 
   
    for n in 0..event.n_paddles {
      match PaddlePacket::from_bytestream(&bytestream, pos) {
        Err(err) => {
          return Err(err);
        }
        Ok(pp)   => {
          event.paddle_packets.push(pp);
          pos += PaddlePacket::PACKETSIZE;
        }
      }
    }
    Ok(event) 
  }
  
  pub fn to_bytestream(&self) -> Vec<u8> {

    let mut bytestream = Vec::<u8>::with_capacity(TofEvent::PacketSizeFixed + (self.n_paddles as usize)*PaddlePacket::PACKETSIZE as usize);

    bytestream.extend_from_slice(&TofEvent::Head.to_le_bytes());
    let mut evid = self.event_id.to_be_bytes();

    evid  = [evid[1],
             evid[0],
             evid[3],
             evid[2]];
    bytestream.extend_from_slice(&evid);
    bytestream.push(self.n_paddles);
    for n in 0..self.n_paddles as usize {
      let pp = self.paddle_packets[n];
      bytestream.extend_from_slice(&pp.to_bytestream());

    }
    bytestream.extend_from_slice(&TofEvent::Tail        .to_le_bytes()); 

    bytestream

  }

  /// Add a paddle packet 
  ///  
  /// This makes sure the internal counter for 
  /// paddles is also incremented.
  pub fn add_paddle(&mut self, paddle : PaddlePacket) -> Result<(), EventError> {
    if self.event_id != paddle.event_id {
      error!("Tried to add paddle for event {} to event{}", self.event_id, paddle.event_id);
      return Err(EventError::EventIdMismatch);
    }
    self.n_paddles += 1;
    self.paddle_packets.push(paddle);
    Ok(())
  }


  ///! Check if a certain time span has passed since event's creation
  ///
  ///
  ///
  pub fn has_timed_out(&self) -> bool {
    return elapsed_since_epoch() - self.creation_time > EVENT_TIMEOUT;
  }

  pub fn is_complete(&self) -> bool {
    self.n_paddles == self.n_paddles_expected
  }

  ///! This means that all analysis is 
  ///  done, and it is fully assembled
  ///
  ///  Alternatively, the timeout has 
  ///  been passed
  ///
  pub fn is_ready_to_send(&self, use_timeout : bool)
    -> bool {
    return self.is_complete() || (self.has_timed_out() && use_timeout);
  }
}


///
/// TESTS
///
/// ============================================

#[test]
fn serialize_deserialize_pp_roundabout() {
  let mut pp = PaddlePacket::from_random();
  // a fresh packet is always valid
  assert!(pp.valid);
  // FIXME - as an idea. If we use
  // 4 byte emoji data, we can easily
  // check if a bytestream is that what
  // we expect visually 
   
  
  //let mut bytestream = Vec<u8>::new();
  let mut bytestream = pp.to_bytestream();
  match PaddlePacket::from_bytestream(&bytestream, 0) {
    Err(err) => {
      error!("Got deserialization error! {:?}", err);
    },
    Ok(new_pp)   => {
      assert_eq!(new_pp, pp);
    }
  }
}

#[test]
fn serialize_deserialize_tofevent_roundabout() {
  let mut event = TofEvent::new(0,0);

  // let's add 10 random paddles
  for n in 0..10 {
    let pp = PaddlePacket::from_random();
    event.paddle_packets.push(pp);
  }
  assert!(event.valid);

  //let mut bytestream = Vec<u8>::new();
  let mut bytestream = event.to_bytestream();
  match TofEvent::from_bytestream(&bytestream, 0) {
    Err(err) => {
      error!("Got deserialization error! {:?}", err);
    },
    Ok(new_event)   => {
      assert_eq!(new_event, event);
    }
  }
}

