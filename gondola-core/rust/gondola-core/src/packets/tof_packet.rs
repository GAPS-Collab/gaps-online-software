//! TofPacket provides a wrapper to write objects which implement
//! TofPackable into files
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Internal Tof communication protocol.
/// Simple, yet powerful
///
/// A TofPacket has the following layout
/// on disk
/// HEAD    : u16 = 0xAAAA
/// TYPE    : u8  = PacketType
/// SIZE    : u32
/// PAYLOAD : [u8;6-SIZE]
/// TAIL    : u16 = 0x5555 
///
/// The total packet size is thus 13 + SIZE
#[derive(Debug, Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofPacket {
  /// Type of the structure encoded in payload
  pub packet_type        : TofPacketType,
  /// The bytestream encoded structure
  pub payload            : Vec<u8>,
  // fields which won't get serialized
  /// mark a packet as not eligible to be written to disk
  pub no_write_to_disk   : bool,
  /// mark a packet as not eligible to be sent over network 
  /// FIXME - future extension
  pub no_send_over_nw    : bool,
  /// creation_time for the instance
  pub creation_time      : Instant,
  pub valid              : bool, // will be always valid, unless invalidated
}

impl fmt::Display for TofPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let p_len = self.payload.len();
    if p_len < 4 {
      write!(f, "<TofPacket: type {:?}, payload size {}>", self.packet_type, p_len)
    } else {
      write!(f, "<TofPacket: type {:?}, payload [ {} {} {} {} .. {} {} {} {}] of size {} >",
             self.packet_type,
             self.payload[0], self.payload[1], self.payload[2], self.payload[3],
             self.payload[p_len-4], self.payload[p_len-3], self.payload[p_len - 2], self.payload[p_len-1], p_len ) 
    }
  }
}

impl Default for TofPacket {
  fn default() -> Self {
    Self::new()
  }
}

/// Implement because TofPacket saves the creation time, 
/// which never will be the same for 2 different instances
impl PartialEq for TofPacket {
  fn eq(&self, other: &Self) -> bool {
    (self.packet_type == other.packet_type)           &&
    (self.payload == other.payload)                   &&
    (self.no_write_to_disk == other.no_write_to_disk) &&
    (self.no_send_over_nw == other.no_send_over_nw)   &&
    (self.valid == other.valid)
  }
}

impl TofPacket {

  pub fn new() -> Self {
    let creation_time = Instant::now();
    Self {
      packet_type      : TofPacketType::Unknown,
      payload          : Vec::<u8>::new(),
      no_write_to_disk : false,
      no_send_over_nw  : false,
      creation_time    : creation_time,
      valid            : true,
    }
  }

  /// Generate a bytestream of self for ZMQ, prefixed with 
  /// BRCT so all RBs will see it
  pub fn zmq_payload_brdcast(&self) -> Vec<u8> {
    let mut payload     = String::from("BRCT").into_bytes(); 
    let mut stream  = self.to_bytestream();
    payload.append(&mut stream);
    payload
  }
  
  /// Generate a bytestream of self for ZMQ, prefixed with 
  /// RBX, to address only a certain board
  pub fn zmq_payload_rb(&self, rb_id : u8) -> Vec<u8> {
    let mut payload     = format!("RB{:02}", rb_id).into_bytes(); 
    let mut stream  = self.to_bytestream();
    payload.append(&mut stream);
    payload
  }

  /// Unpack the TofPacket and return its content
  pub fn unpack<T>(&self) -> Result<T, SerializationError>
    where T: TofPackable + Serialization {
    // if the alternative tof packet type is unknown, it must 
    // be of type TOF_PACKET_TYPE, alternatively either one 
    // of Both
    if T::TOF_PACKET_TYPE_ALT != TofPacketType::Unknown {
      if T::TOF_PACKET_TYPE_ALT == self.packet_type {
        let unpacked : T = T::from_bytestream_alt(&self.payload, &mut 0)?;
        return Ok(unpacked); 
      } else if T::TOF_PACKET_TYPE == self.packet_type{
        let unpacked : T = T::from_bytestream(&self.payload, &mut 0)?;
        return Ok(unpacked); 
      } else {
        error!("This packet of type {} is neither for a {} nor a {}  packet!", self.packet_type, T::TOF_PACKET_TYPE, T::TOF_PACKET_TYPE_ALT);
        return Err(SerializationError::IncorrectPacketType); 
      }
    } else {
      if T::TOF_PACKET_TYPE != self.packet_type {
        error!("This bytestream is not for a {} packet!", self.packet_type);
        return Err(SerializationError::IncorrectPacketType);
      }
    }
    let unpacked : T = T::from_bytestream(&self.payload, &mut 0)?;
    Ok(unpacked)
  }
  
  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }
}


impl Serialization for TofPacket {
  const HEAD : u16 = 0xaaaa;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 0; // FIXME - size/prelude_size 

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
  -> Result<Self, SerializationError> {
    if stream.len() < 2 {
      return Err(SerializationError::StreamTooShort);
    }
    let head = parse_u16(stream, pos);
    if Self::HEAD != head {
      error!("TofPacket does not start with HEAD signature! {}", Self::HEAD);
      return Err(SerializationError::HeadInvalid);
    }
    let packet_type : TofPacketType;
    let packet_type_enc = parse_u8(stream, pos);
    match TofPacketType::try_from(packet_type_enc) {
      Ok(pt) => packet_type = pt,
      Err(_) => {
        error!("Can not decode packet with packet type {}", packet_type_enc);
        return Err(SerializationError::UnknownPayload);}
    }
    let payload_size = parse_u32(stream, pos) as usize;
    *pos += payload_size; 
    let tail = parse_u16(stream, pos);
    if Self::TAIL != tail {
      error!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid);
    }
    *pos -= 2; // for tail parsing
    *pos -= payload_size;

    let mut tp = TofPacket::new();
    tp.packet_type = packet_type;
    tp.payload.extend_from_slice(&stream[*pos..*pos+payload_size]);
    // Fix position marker
    *pos += 2 + payload_size;
    Ok(tp) 
  }
  
  fn to_bytestream(&self) 
    -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(6 + self.payload.len());
    bytestream.extend_from_slice(&TofPacket::HEAD.to_le_bytes());
    let p_type = self.packet_type as u8;
    bytestream.push(p_type);
    // payload size of 32 bit accomodates up to 4 GB packet
    // a 16 bit size would only hold 65k, which might be not
    // good enough if we sent multiple events in a batch in 
    // the same TofPacket (in case we do that)
    let payload_len = self.payload.len() as u32;
    //let foo = &payload_len.to_le_bytes();
    //debug!("TofPacket binary payload: {foo:?}");
    bytestream.extend_from_slice(&payload_len.to_le_bytes());
    bytestream.extend_from_slice(self.payload.as_slice());
    bytestream.extend_from_slice(&TofPacket::TAIL.to_le_bytes());
    bytestream
  }
}

#[cfg(feature="random")]
impl FromRandom for TofPacket {

  fn from_random() -> Self {
    // FIXME - this should be an actual, realistic
    // distribution
    let choices = [
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::RBWaveform,
      TofPacketType::RBWaveform,
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::TofEvent,
      TofPacketType::MasterTrigger,
      TofPacketType::MasterTrigger,
      TofPacketType::MasterTrigger,
      TofPacketType::RBMoniData,
      TofPacketType::PBMoniData,
      TofPacketType::LTBMoniData,
      TofPacketType::PAMoniData,
      TofPacketType::CPUMoniData,
      TofPacketType::MtbMoniData,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    let packet_type = choices[idx];
    match packet_type {
      TofPacketType::TofEvent => {
        let te = TofEvent::from_random();
        return te.pack()
      }
      TofPacketType::RBWaveform => {
        let te = RBWaveform::from_random();
        return te.pack()
      }
      TofPacketType::RBMoniData => {
        let te = RBMoniData::from_random();
        return te.pack()
      }
      TofPacketType::PAMoniData => {
        let te = PAMoniData::from_random();
        return te.pack()
      }
      TofPacketType::LTBMoniData => {
        let te = LTBMoniData::from_random();
        return te.pack()
      }
      TofPacketType::PBMoniData => {
        let te = PBMoniData::from_random();
        return te.pack()
      }
      TofPacketType::CPUMoniData => {
        let te = CPUMoniData::from_random();
        return te.pack()
      }
      TofPacketType::MtbMoniData  => {
        let te = MtbMoniData::from_random();
        return te.pack()
      }
      _ => {
        let te = TofEvent::from_random();
        return te.pack()
      }
    }
  }
}

impl Frameable for TofPacket {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType = CRFrameObjectType::TofPacket;
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofPacket {

  #[getter]
  fn get_packet_type(&self) -> TofPacketType {
    self.packet_type
  }
 
  // FIXME - trust in te process that it referenceces te input vector and not clones it
  /// Factory function for TofPackets
  ///
  /// # Arguments:
  ///
  ///   * stream    : bytes presumably representing
  ///                 a TofPacket
  ///   * start_pos : the assumed position of 
  ///                 HEAD identifier in the
  ///                 bytestream (start of 
  ///                 TofPacket)
  #[staticmethod]
  #[pyo3(name = "from_bytestream")]
  fn from_bytestream_py<'_py>(stream : Vec<u8>, start_pos : usize) -> PyResult<Self>{
    let mut pos = start_pos;  
    match Self::from_bytestream(&stream, &mut pos) {
      Ok(tp) => {
        return Ok(tp);
      }
      Err(err) => {
        let err_msg = format!("Unable to TofPacket from bytestream! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  #[pyo3(name="unpack")]
  fn unpack_py(&self,py: Python) -> PyResult<Py<PyAny>> {
    match self.packet_type {
      TofPacketType::Unknown               => {
        let msg = "TofPacket is of type 'Unknown' and thus can't be unpacked!";
        return Err(PyValueError::new_err(msg));
      }, 
      TofPacketType::RBEvent               => {
        match self.unpack::<RBEvent>() {
          Ok(data) => {
            return Ok(Py::new(py, data)?.into_any());
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }, 
      TofPacketType::TofEventDeprecated  => {
        match self.unpack::<TofEvent>() {
          Ok(data) => {
            return Ok(Py::new(py, data)?.into_any());
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }, 
      TofPacketType::RBWaveform               => {
        match self.unpack::<RBWaveform>() {
          Ok(data) => {
            return Ok(Py::new(py, data)?.into_any());
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }, 
      TofPacketType::TofEvent               => {
        match self.unpack::<TofEvent>() {
          Ok(data) => {
            return Ok(Py::new(py, data)?.into_any());
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }, 
      TofPacketType::DataSinkHB               => {
        match self.unpack::<DataSinkHB>() {
          Ok(data) => {
            return Ok(Py::new(py, data)?.into_any());
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }, 
      //TofPacketType::MasterTrigger         => {}, 
      //TofPacketType::TriggerConfig         => {},
      //TofPacketType::MasterTriggerHB       => {}, 
      //TofPacketType::EventBuilderHB        => {},
      //TofPacketType::RBChannelMaskConfig   => {},
      //TofPacketType::TofRBConfig           => {},
      //TofPacketType::AnalysisEngineConfig  => {},
      //TofPacketType::RBEventHeader         => {},    
      //TofPacketType::TOFEventBuilderConfig => {},
      //TofPacketType::DataPublisherConfig   => {},
      //TofPacketType::TofRunConfig          => {},
      //TofPacketType::CPUMoniData           => {},
      //TofPacketType::MtbMoniData           => {},
      //TofPacketType::RBMoniData            => {},
      //TofPacketType::PBMoniData            => {},
      //TofPacketType::LTBMoniData           => {},
      //TofPacketType::PAMoniData            => {},
      //TofPacketType::RBEventMemoryView     => {}, 
      //TofPacketType::RBCalibration         => {},
      //TofPacketType::TofCommand            => {},
      //TofPacketType::TofCommandV2          => {},
      //TofPacketType::TofResponse           => {},
      //TofPacketType::RBCommand             => {},
      //TofPacketType::RBPing                => {},
      //TofPacketType::PreampBiasConfig      => {},
      //TofPacketType::RunConfig             => {},
      //TofPacketType::LTBThresholdConfig    => {},
      //TofPacketType::TofDetectorStatus     => {},
      //TofPacketType::ConfigBinary          => {},
      //TofPacketType::LiftofRBBinary        => {},
      //TofPacketType::LiftofBinaryService   => {},
      //TofPacketType::LiftofCCBinary        => {},
      //TofPacketType::RBCalibrationFlightV  => {},
      //TofPacketType::RBCalibrationFlightT  => {},
      //TofPacketType::BfswAckPacket         => {},
      //TofPacketType::MultiPacket           => {},
      _               => {
        match self.unpack::<TofEvent>() {
          Ok(data) => {
            return Ok(Py::new(py, data)?.into_any());
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }, 
    }
  }
}

#[cfg(feature="pybindings")]
pythonize!(TofPacket);

