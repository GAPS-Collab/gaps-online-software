/****
 *
 * Communications with the 
 * mastertrigger
 *
 */ 

// to measure the rate
use std::time::{Duration, Instant};
use std::thread;
use std::net::{UdpSocket, SocketAddr};
use std::sync::mpsc::Sender;

use std::io;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::master_trigger::{read_daq, read_rate, reset_daq};
use tof_dataclasses::events::MasterTriggerEvent;

use liftof_lib::connect_to_mtb;

//const MT_MAX_PACKSIZE   : usize = 4096;



