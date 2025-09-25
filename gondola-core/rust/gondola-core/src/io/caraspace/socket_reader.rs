/// ZMQ socket wrapper for the zmq socket which is 
/// supposed to receive data from the TOF system.
pub fn socket_wrap_tofstream(address   : &str,
                             tp_sender : Sender<TofPacket>) {
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

//use telemetry_dataclasses::packets::{
//  TelemetryPacketHeader,
//  TelemetryPacket,
//};

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

