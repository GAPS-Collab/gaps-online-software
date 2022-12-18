/****
 *
 * Communications with the 
 * mastertrigger
 *
 */ 


use std::net::UdpSocket;

///
/// Communications with the master trigger
///
///
pub fn master_and_commander(mt_ip   : &str, 
                            mt_port : usize) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
  // FIXME - proper error checking
  let socket = UdpSocket::bind(mt_address).unwrap();
  let mut buffer : [u8;4096]; // the packet size might be 
                              // 4096 bytes
  buffer = [0;4096];
  loop {
    let received = socket.recv_from(&mut buffer);

    match received {
      Ok((size, addr)) => println!("Received {} bytes from address {}", size, addr),
      Err(err)         => {
        println!("Received nothing! err {}", err);
        continue;
      }
    } // end match
  } // end loop
}

