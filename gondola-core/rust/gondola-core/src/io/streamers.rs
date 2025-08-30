//! 
//! * RBEventMemoryStreamer: Walk over "raw" RBEvents
//!   representations ("RBEventMemoryView") and extract
//!   RBEvents
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use crossbeam_channel::Sender;

/// ZMQ socket wrapper for the zmq socket which is 
/// supposed to receive data from the TOF system.
pub fn socket_wrap_tofstream(address   : &str,
                             tp_sender : Sender<TofPacket>) {
  // FIXME - would be nice to make this generic, but 
  //         Telemetry and Tofstream are too different 
  let ctx = zmq::Context::new();
  // FIXME - don't hardcode this IP
  let socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  socket.connect(address).expect("Unable to connect to data (PUB) socket {adress}");
  socket.set_subscribe(b"").expect("Can't subscribe to any message on 0MQ socket!");
  //let mut n_pack = 0usize;
  info!("0MQ SUB socket connected to address {address}");
  // per default, we create master trigger packets from TofEventSummary, 
  // except we have "real" mtb packets
  //let mut craft_mte_packets = true;
  loop {
    match socket.recv_bytes(0) {
      Err(err) => error!("Can't receive TofPacket! {err}"),
      Ok(payload)    => {
        match TofPacket::from_bytestream(&payload, &mut 0) {
          Ok(tp) => {
            match tp_sender.send(tp) {
              Ok(_) => (),
              Err(err) => error!("Can't send TofPacket over channel! {err}")
            }
          }
          Err(err) => {
            debug!("Can't decode payload! {err}");
            // that might have an RB prefix, forward 
            // it 
            match TofPacket::from_bytestream(&payload, &mut 4) {
              Err(err) => {
                error!("Don't understand bytestream! {err}"); 
              },
              Ok(tp) => {
                match tp_sender.send(tp) {
                  Ok(_) => (),
                  Err(err) => error!("Can't send TofPacket over channel! {err}")
                }
              }
            }
          }  
        }
      }
    }
  }
}

/// Get the GAPS merged event telemetry stream and 
/// broadcast it to the relevant tab
///
/// # Arguments
///
/// * address     : Address to susbscribe to for telemetry 
///                 stream (must be zmq.PUB) on the Sender
///                 side
/// * cachesize   : Getting the packets from the funneled stream leads
///                 to duplicates. To eliminate these, we store the 
///                 packet counter variable in a Dequee of a given 
///                 size
/// * tele_sender : Channel to forward the received telemetry
///                 packets
pub fn socket_wrap_telemetry(address     : &str,
                             cachesize   : usize,
                             tele_sender : Sender<TelemetryPacket>) {
  let ctx = zmq::Context::new();
  // FIXME - don't hardcode this IP
  // typically how it is done is that this program runs either on a gse
  // or there is a local forwarding of the port thrugh ssh
  //let address : &str = "tcp://127.0.0.1:55555";
  let socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  match socket.connect(&address) {
    Err(err) => {
      error!("Unable to connect to data (PUB) socket {address}! {err}");
      panic!("Can not connect to zmq PUB socket!");
    }
    Ok(_) => ()
  }
  let mut cache = VecDeque::<u16>::with_capacity(cachesize);
  socket.set_subscribe(b"") .expect("Can't subscribe to any message on 0MQ socket! {err}");
  loop {
    match socket.recv_bytes(0) {
      Err(err)    => error!("Can't receive TofPacket! {err}"),
      Ok(mut payload) => {
        match TelemetryPacketHeader::from_bytestream(&payload, &mut 0) {
          Err(err) => {
            error!("Can not decode telemtry header! {err}");
            //for k in pos - 5 .. pos + 5 {
            //  println!("{}",stream[k]);
            //}
          }
          Ok(header) => {
            let mut packet = TelemetryPacket::new();
            if payload.len() > TelemetryPacketHeader::SIZE {
              payload.drain(0..TelemetryPacketHeader::SIZE);
            }
            if cache.contains(&header.counter) {
              // drop this packet
              continue;
            } else {
              cache.push_back(header.counter); 
            }
            if cache.len() == cachesize {
              cache.pop_front();
            }

            packet.header  = header;
            packet.payload = payload;
            match tele_sender.send(packet) {
              Err(err) => error!("Can not send telemetry packet to downstream! {err}"),
              Ok(_)    => ()
            }
          }
        }
      }
    }
  }
}




// only used here
use crc::Crc;

// change if we switch to a firmware
// where the byteorder of u32 and larger 
// is correct.
const REVERSE_WORDS : bool = true;
const ALGO : crc::Algorithm<u32> = crc::Algorithm {
      width   : 32u8,
      init    : 0xFFFFFFFF,
      //poly    : 0xEDB88320,
      poly    : 0x04C11DB7,
      refin   : true,
      refout  : true,
      xorout  : 0xFFFFFFFF,
      check   : 0,
      residue : 0,
    };

/// Emit RBEvents from a stream of bytes
/// from RBMemory
///
/// The layout of the stream has to have
/// the fpga fw memory layout.
///
/// This provides a next() method to act
/// as a generator for RBEvents
pub struct RBEventMemoryStreamer {
  /// Raw stream read out from the RB buffers.
  pub stream               : Vec<u8>,
  /// Error checking mode - check error bits for 
  /// channels/cells
  pub check_channel_errors : bool,
  /// Ignore channels in this list
  pub mask                 : Vec<u8>,

  /// Current position in the stream
  pos                      : usize,
  /// The current posion marker points to a header 
  /// signature in the stream.
  pos_at_head              : bool,
  /// An optional crossbeam::channel Sender, which 
  /// will allow to send TofPackets
  pub tp_sender            : Option<Sender<TofPacket>>,
  /// number of extracted events from stream
  /// this manages how we are draining the stream
  n_events_ext             : usize,
  pub is_depleted          : bool,
  /// Calculate the crc32 checksum for the channels 
  /// everytime next() is called
  pub calc_crc32           : bool,
  /// placeholder for checksum calculator
  crc32_sum                : Crc::<u32>,
  pub request_mode         : bool,
  pub request_cache        : VecDeque<(u32,u8)>,
  /// an index for the events in the stream
  /// this links eventid and start position
  /// in the stream together
  pub event_map            : HashMap<u32,(usize,usize)>,
  pub first_evid           : u32,
  pub last_evid            : u32,
  pub last_event_complete  : bool,
  pub last_event_pos       : usize,
  /// When in request mode, number of events the last event in the stream is behind the
  /// first request
  pub is_behind_by         : usize,
  /// When in request mode, number of events the last event in the stream is ahead the
  /// last request
  pub is_ahead_by          : usize,
}

impl RBEventMemoryStreamer {

  pub fn new() -> Self {
    Self {
      stream               : Vec::<u8>::new(),
      check_channel_errors : false,
      mask                 : Vec::<u8>::new(),
      pos                  : 0,
      pos_at_head          : false,
      tp_sender            : None,
      n_events_ext         : 0,
      is_depleted          : false,
      calc_crc32           : false,
      crc32_sum            : Crc::<u32>::new(&ALGO),
      request_mode         : false,
      request_cache        : VecDeque::<(u32,u8)>::new(),
      event_map            : HashMap::<u32,(usize,usize)>::new(),
      first_evid           : 0,
      last_evid            : 0,
      last_event_complete  : false,
      last_event_pos       : 0,
      is_behind_by         : 0,
      is_ahead_by          : 0,
    }
  }
 
  /// Create the event index, which is
  /// a map of event ids and position 
  /// + length in the stream
  pub fn create_event_index(&mut self) { //-> Result<Ok, SerializationError> {
    let begin_pos = self.pos;
    let mut event_id = 0u32;
    // we are now at head, 
    // read packet len and event id
    loop {
      let mut result = (0usize, 0usize);
      if !self.seek_next_header(0xaaaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        self.last_evid = event_id;
        if result.0 + result.1 > self.stream.len() - 1 {
          self.last_event_complete = false;
        } else {
          self.last_event_complete = true;
        }
        info!("Indexed {} events from {} to {}", self.event_map.len(), self.first_evid, self.last_evid);
        return;
      }
      result.0 = self.pos;
      self.pos += 4;//header, status
      let packet_len = parse_u16(&self.stream, &mut self.pos) as usize * 2;
      if self.stream.len() < self.pos -6 + packet_len {
        //self.is_depleted = true;
        self.pos = begin_pos;
        self.last_evid = event_id;
        info!("Indexed {} events from {} to {}", self.event_map.len(), self.first_evid, self.last_evid);
        return;
        //return Err(SerializationError::StreamTooShort);
      }
      result.1 = packet_len;
      if packet_len < 6 {
        self.pos = begin_pos;
        self.last_evid = event_id;
        info!("Indexed {} events from {} to {}", self.event_map.len(), self.first_evid, self.last_evid);
        return;
        //return Err(SerializationError::StreamTooShort);
      }
      // rewind
      self.pos -= 6;
      // event id is at pos 22
      self.pos += 22;
      let event_id0    = parse_u16(&self.stream, &mut self.pos) as u32;
      let event_id1    = parse_u16(&self.stream, &mut self.pos) as u32;
      if REVERSE_WORDS {
        event_id = event_id0 << 16 | event_id1;
      } else {
        event_id = event_id1 << 16 | event_id0;
      }
      if self.first_evid == 0 {
        self.first_evid = event_id;
      }
      self.pos += packet_len - 26;
      self.event_map.insert(event_id,result);
    }
  }

  pub fn print_event_map(&self) {
    for k in self.event_map.keys() {
      let pos = self.event_map[&k];
      println!("-- --> {} -> {},{}", k, pos.0, pos.1);
    }
  }

  // EXPERIMENTAL
  pub fn init_sender(&mut self, tp_sender : Sender<TofPacket>) {
    self.tp_sender = Some(tp_sender);
  }

  // EXPERIMENTAL
  pub fn send_all(&mut self) {
    loop {
      match self.next() {
        None => {
          info!("Streamer drained!");
          break;
        },
        Some(ev) => {
          let tp = ev.pack();
          match self.tp_sender.as_ref().expect("Sender needs to be initialized first!").send(tp) {
            Ok(_) => (),
            Err(err) => {
              error!("Unable to send TofPacket! {err}");
            }
          }
        }
      }
    }
  }


  // FIXME - performance. Don't extend it. It would be
  // better if we'd consume the stream without 
  // reallocating memory.
  pub fn add(&mut self, stream : &Vec<u8>, nbytes : usize) {
    //self.stream.extend(stream.iter().copied());
    //println!("self.pos {}", self.pos);
    //println!("Stream before {}",self.stream.len());
    self.is_depleted = false;
    self.stream.extend_from_slice(&stream[0..nbytes]);
    //self.create_event_index();
    //println!("Stream after {}",self.stream.len());
  }

  /// Take in a stream by consuming it, that means moving
  /// This will avoid clones.
  pub fn consume(&mut self, stream : &mut Vec<u8>) {
    self.is_depleted = false;
    // FIXME: append can panic
    // we use it here, since it does not clone
    //println!("[io.rs] consuming {} bytes", stream.len());
    self.stream.append(stream);
    //println!("[io.rs] stream has now {} bytes", self.stream.len());
    //self.create_event_index();
  }

  /// Headers are expected to be a 2byte signature, 
  /// e.g. 0xaaaa. 
  ///
  /// # Arguments:
  ///   header : 2byte header.
  ///
  /// # Returns
  /// 
  ///   * success   : header found
  pub fn seek_next_header(&mut self, header : u16) -> bool{
    match seek_marker(&self.stream, header, self.pos) { 
    //match search_for_u16(header, &self.stream, self.pos) {
      Err(_) => {
        return false;
      }
      Ok(head_pos) => {
        self.pos = head_pos;
        self.pos_at_head = true;
        return true;
      }
    }
  }

  pub fn next_tofpacket(&mut self) -> Option<TofPacket> {
    let begin_pos = self.pos; // in case we need
                              // to reset the position
    let foot_pos : usize;
    let head_pos : usize;
    if self.stream.len() == 0 {
      trace!("Stream empty!");
      return None;
    }
    if !self.pos_at_head {
      if !self.seek_next_header(0xaaaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        return None;
      }
    }
    head_pos  = self.pos;
    //let mut foot_pos  = self.pos;
    //head_pos = self.pos;
    if !self.seek_next_header(0x5555) {
      debug!("Could not find another footer...");
      self.pos = begin_pos;
      return None;
    }
    //println!("{} {} {}", self.stream.len(), head_pos, foot_pos);
    foot_pos = self.pos;
    self.n_events_ext += 1;
    let mut tp = TofPacket::new();
    tp.packet_type = TofPacketType::RBEventMemoryView;
    //let mut payload = Vec::<u8>::with_capacity(18530);
    tp.payload.extend_from_slice(&self.stream[head_pos..foot_pos+2]);
    //tp.payload = payload;
    //self.pos += 2;
    self.pos_at_head = false;
    //self.stream.drain(0..foot_pos);
    //self.pos = 0;
    if self.n_events_ext % 200 == 0 {
      self.stream.drain(0..foot_pos+3);
      self.pos = 0;
    }
    Some(tp)
  }


  /// Retrive an RBEvent from a certain position
  pub fn get_event_at_pos_unchecked(&mut self,
                                    replace_channel_mask : Option<u16>)
      -> Option<RBEvent> {
    let mut header       = RBEventHeader::new();
    let mut event        = RBEvent::new();
    let mut event_status = EventStatus::Unknown;
    //let begin_pos = self.pos;
    if self.calc_crc32 && self.check_channel_errors {
      event_status = EventStatus::Perfect;
    }
    if !self.calc_crc32 && !self.check_channel_errors {
      event_status = EventStatus::GoodNoCRCOrErrBitCheck;
    }
    if !self.calc_crc32 && self.check_channel_errors {
      event_status = EventStatus::GoodNoCRCCheck;
    }
    if self.calc_crc32 && !self.check_channel_errors {
      event_status = EventStatus::GoodNoErrBitCheck;
    }
    // start parsing
    //let first_pos = self.pos;
    let head   = parse_u16(&self.stream, &mut self.pos);
    if head != RBEventHeader::HEAD {
      error!("Event does not start with {}", RBEventHeader::HEAD);
      return None;
    }

    let status = parse_u16(&self.stream, &mut self.pos);
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    header.parse_status(status);
    let packet_len = parse_u16(&self.stream, &mut self.pos) as usize * 2;
    let nwords     = parse_u16(&self.stream, &mut self.pos) as usize + 1; // the field will tell you the 
    if self.pos - 8 + packet_len > self.stream.len() { // -1?
      error!("Stream is too short! Stream len is {}, packet len is {}. We are at pos {}", self.stream.len(), packet_len, self.pos);
      self.is_depleted = true;
      self.pos -= 8;
      return None;
    }
    // now we skip the next 10 bytes, 
    // they are dna, rsv, rsv, rsv, fw_hash
    self.pos += 10;
    self.pos += 1; // rb id first byte is rsvd
    header.rb_id        = parse_u8(&self.stream, &mut self.pos);
    header.set_channel_mask(parse_u16(&self.stream, &mut self.pos)); 
    match replace_channel_mask {
      None => (),
      Some(mask) => {
        println!("==> Replacing ch mask {} with {}", header.get_channel_mask(), mask);
        header.set_channel_mask(mask); 
      }
    }
    let event_id0       = parse_u16(&self.stream, &mut self.pos) as u32;
    let event_id1       = parse_u16(&self.stream, &mut self.pos) as u32;
    let event_id : u32;
    if REVERSE_WORDS {
      event_id = event_id0 << 16 | event_id1;
    } else {
      event_id = event_id1 << 16 | event_id0;
    }
    
    header.event_id  = event_id;
    // we are currently not using these
    //let _dtap0       = parse_u16(&self.stream, &mut self.pos);
    //let _drs4_temp   = parse_u16(&self.stream, &mut self.pos);
    self.pos += 4;
    let timestamp0   = parse_u16(&self.stream, &mut self.pos);
    let timestamp1   = parse_u16(&self.stream, &mut self.pos) as u32;
    let timestamp2   = parse_u16(&self.stream, &mut self.pos);
    //println!("TIMESTAMPS {} {} {}", timestamp0, timestamp1, timestamp2);
    let timestamp16 : u16;
    let timestamp32 : u32;
    if REVERSE_WORDS {
      timestamp16 = timestamp0;
      timestamp32 = timestamp1 << 16 | timestamp2 as u32;
    } else {
      timestamp16 = timestamp2;
      timestamp32 = (timestamp0 as u32) << 16 | timestamp1;
    }
    header.timestamp16 = timestamp16;
    header.timestamp32 = timestamp32;
    // now the payload
    //println!("{}", header);
    //println!("{}", nwords);
    if header.drs_lost_trigger() {
      event.status = EventStatus::IncompleteReadout;
      event.header = header;
      //self.pos_at_head = false;
      return Some(event);
    }
    // make sure we can read them!
    //let expected_packet_size =   header.get_channels().len()*nwords*2 
    //                           + header.get_channels().len()*2 
    //                           + header.get_channels().len()*4;
    let mut any_cell_error = false;
    let mut header_channels = header.get_channels().clone();
    for k in &self.mask {
      header_channels.retain(|x| x != k);
    }

    for ch in header_channels.iter() {
      let ch_id = parse_u16(&self.stream, &mut self.pos);
      if ch_id != *ch as u16 {
        // check where is the next header
        let search_pos = self.pos;
        match seek_marker(&self.stream, TofPacket::HEAD, search_pos) { 
        //match search_for_u16(TofPacket::HEAD, &self.stream, search_pos) {
          Err(_) => (),
          Ok(result) => {
            info!("The channel data is corrupt, but we found a header at {} for remaining stream len {}", result, self.stream.len()); 
          }
        }
        let mut stream_view = Vec::<u8>::new();
        let foo_pos = self.pos;
        for k in foo_pos -3..foo_pos + 3 {
          stream_view.push(self.stream[k]);
        }
        error!("We got {ch_id} but expected {ch} for event {}. The parsed ch id is not in the channel mask! We will fill this channel with u16::MAX .... Stream view +- 3 around the ch id {:?}", header.event_id, stream_view);
        event_status = EventStatus::ChannelIDWrong;
        // we fill the channel with MAX values:
        event.adc[*ch as usize] = vec![u16::MAX;NWORDS];
        self.pos += 2*nwords + 4;
        continue;
      } else {
      //if ch_id == *ch as u16 {
        //println!("Got ch id {}", ch_id);
        //let header = parse_u16(&self.stream, &mut self.pos);
        // noice!!
        //let data : Vec<u8> = self.stream.iter().skip(self.pos).take(2*nwords).map(|&x| x).collect();
         
        let mut dig = self.crc32_sum.digest();
        if self.calc_crc32 {
          let mut this_ch_adc = Vec::<u16>::with_capacity(nwords);
          for _ in 0..nwords {
            let this_field = parse_u16(&self.stream, &mut self.pos);
            dig.update(&this_field.to_le_bytes());
            if self.check_channel_errors {
              if ((0x8000 & this_field) >> 15) == 0x1 {
                error!("Ch error bit set for ch {}!", ch);
                event_status = EventStatus::ChnSyncErrors;
              }
              if ((0x4000 & this_field) >> 14) == 0x1 {
                error!("Cell error bit set for ch {}!", ch);
                event_status = EventStatus::CellSyncErrors;
                any_cell_error = true;
              }
            }
            this_ch_adc.push(0x3fff & this_field)
          }
          event.adc[*ch as usize] = this_ch_adc;
        } else {
          if self.check_channel_errors {
            let adc_w_errs = u8_to_u16_err_check(&self.stream[self.pos..self.pos + 2*nwords]);
            if adc_w_errs.1 {
              error!("Ch error bit set for ch {}!", ch);
              event_status = EventStatus::ChnSyncErrors;
              any_cell_error = true;
            } else if adc_w_errs.2 {
              error!("Cell error bit set for ch {}!", ch);
              event_status = EventStatus::CellSyncErrors;
            }
            event.adc[*ch as usize] = adc_w_errs.0;
          } else {
            event.adc[*ch as usize] = u8_to_u16_14bit(&self.stream[self.pos..self.pos + 2*nwords]);
          }
          self.pos += 2*nwords;
        } 
        //let data = &self.stream[self.pos..self.pos+2*nwords];
        //self.pos += 2*nwords;
        let crc320 = parse_u16(&self.stream, &mut self.pos) as u32;
        let crc321 = parse_u16(&self.stream, &mut self.pos) as u32;
        //let checksum = self.crc32_sum.clone().finalize();
        if self.calc_crc32 {
          let crc32 : u32;
          if REVERSE_WORDS {
            crc32 = crc320 << 16 | crc321;
          } else {
            crc32 = crc321 << 16 | crc320;
          }
          let checksum = dig.finalize();
          if checksum != crc32 {
            event_status = EventStatus::CRC32Wrong;
          }
          println!("== ==> Checksum {}, channel checksum {}!", checksum, crc32); 
        }
      }
    }
    if any_cell_error {
      if event_status == EventStatus::ChnSyncErrors {
        event_status = EventStatus::CellAndChnSyncErrors;
      }
    }
    
    if !header.drs_lost_trigger() {
      header.stop_cell = parse_u16(&self.stream, &mut self.pos);
    }
    // CRC32 checksum - next 4 bytes
    // FIXME
    // skip crc32 checksum
    self.pos += 4;

    // in principle there is a checksum for the whole event, whcih
    // we are currently not using (it is easy to spot wrong bytes
    // in the header)
    //let crc320         = parse_u16(&self.stream, &mut self.pos);
    //let crc321         = parse_u16(&self.stream, &mut self.pos);
    //if self.calc_crc32 {
    //  let crc32 : u32;
    //  if REVERSE_WORDS {
    //    crc32 = u32::from(crc320) << 16 | u32::from(crc321);
    //  } else {
    //    crc32 = u32::from(crc321) << 16 | u32::from(crc320);
    //  }
    //  warn!("Checksum test for the whole event is not yet implemented!");
    //  //if event.header.crc32 != crc32 {
    //  //  trace!("Checksum test for the whole event is not yet implemented!");
    //  //}
    //}
    
    let tail         = parse_u16(&self.stream, &mut self.pos);
    if tail != 0x5555 {
      error!("Tail signature {} for event {} is invalid!", tail, header.event_id);
      event_status = EventStatus::TailWrong;
    } 
    //self.stream.drain(0..self.pos);
    self.pos_at_head = false;
    event.header = header;
    event.status = event_status;
    if event_status == EventStatus::TailWrong {
      info!("{}", event);
    }
    Some(event)
  }

  pub fn get_event_at_id(&mut self, event_id : u32, replace_channel_mask : Option<u16>) -> Option<RBEvent> {
    let begin_pos = self.pos; // in case we need
                              // to reset the position
    //println!("--> Requested {}", event_id);
    //if self.event_map.contains_key(&event_id) {
    //  //println!("-- We have it!");
    //} else {
    //  //println!("-- We DON'T have it, event_map len {}", self.event_map.len());
    //  //self.print_event_map();
    //  //println!("-- last event id {}", self.last_evid);
    //  //println!("-- first event id {}", self.first_evid);
    //}
    let pos = self.event_map.remove(&event_id)?;
    if self.stream.len() < pos.0 + pos.1 {
      trace!("Stream is too short!");
      self.is_depleted = true;
      self.pos = begin_pos;
      return None;
    }
    self.pos = pos.0;
    self.get_event_at_pos_unchecked(replace_channel_mask)
  }
}

impl Iterator for RBEventMemoryStreamer {
  type Item = RBEvent;

  fn next(&mut self) -> Option<Self::Item> {
    // FIXME - we should init this only once
    // event id from stream
    //let event_id  = 0u32;
    let begin_pos : usize; // in case we need
                           // to rewind
     
    self.pos_at_head = false;
    begin_pos = self.pos; // in case we need
                                // to reset the position
    if self.stream.len() == 0 {
      trace!("Stream empty!");
      self.is_depleted = true;
      self.pos = 0;
      return None;
    }
    if !self.pos_at_head {
      if !self.seek_next_header(0xaaaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        self.is_depleted = true;
        return None;
      }
    }
    
    let event          = self.get_event_at_pos_unchecked(None)?;
    self.n_events_ext += 1;
    self.stream.drain(0..self.pos);
    self.pos           = 0;
    self.pos_at_head   = false;
    Some(event)
  }
}


