pub mod registers;
pub mod memory;
pub mod control;
pub mod api;

use std::{thread, time};

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};

use std::net::IpAddr;

use local_ip_address::local_ip;

//use std::collections::HashMap;

use crate::api::*;
use crate::control::*;
use crate::memory::{BlobBuffer,
                    UIO1_MAX_OCCUPANCY,
                    UIO2_MAX_OCCUPANCY,
                    UIO1_MIN_OCCUPANCY,
                    UIO2_MIN_OCCUPANCY};
use tof_dataclasses::commands::*;

use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::packets::generic_packet::GenericPacket;
use tof_dataclasses::events::blob::RBEventPayload;
use tof_dataclasses::commands::{TofCommand,
                                TofResponse,
                                TofOperationMode};
use tof_dataclasses::commands as cmd;
use tof_dataclasses::monitoring as moni;
use tof_dataclasses::serialization::Serialization;
//use liftof_lib::misc::*;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use log::{info, LevelFilter};
use std::io::Write;

/// The 0MQ PUB port is defined as DATAPORT_START + readoutboard_id
const DATAPORT_START : u32 = 30000;

/// The 0MP REP port is defined as CMDPORT_START + readoutboard_id
const CMDPORT_START  : u32 = 40000;

extern crate clap;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

