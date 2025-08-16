//! This files provides an event structure to pack 
//! and bookkeep RB data
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;


/// Get the traces for a set of RBEvents
///
/// This will return a cube of 
/// The sice of this cube will be fixed
/// in two dimensions, but not the third
///
/// The rationale of this is to be able 
/// to quickly calculate means over all
/// channels.
///
/// Shape
/// \[ch:9\]\[nevents\]\[adc_bin:1024\]
///
/// # Args:
///   events - events to get the traces from
pub fn unpack_traces<T>(events : &Vec<RBEvent>)
  -> Vec<Vec<Vec<T>>> 
  where T: Float + NumAssign + NumCast + Copy {
  let nevents          = events.len();
  let mut traces       = Vec::<Vec::<Vec::<T>>>::new();
  let mut trace        = Vec::<T>::with_capacity(NWORDS);
  let mut stop_cells   = Vec::<isize>::new();
  let mut empty_events = Vec::<Vec::<T>>::new();
  for _ in 0..nevents {
    empty_events.push(trace.clone());
  }
  for ch in 0..NCHN {
    traces.push(empty_events.clone());
    for (k,ev) in events.iter().enumerate() {
      trace.clear();
      stop_cells.push(ev.header.stop_cell as isize);
      for k in 0..NWORDS {
        // the unwrap here can be debated. Technically it does 
        // only ensure that the cast can be possible
        trace.push(T::from(ev.adc[ch][k]).unwrap());
      }
      traces[ch][k] = trace.clone();
    }
  }
  traces
}


/// Event data for each individual ReadoutBoard (RB)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBEvent {
  pub data_type    : DataType,
  pub status       : EventStatus,
  pub header       : RBEventHeader,
  pub adc          : Vec<Vec<u16>>,
  pub hits         : Vec<TofHit>,
}

impl RBEvent {

  pub fn new() -> Self {
    let mut adc = Vec::<Vec<u16>>::with_capacity(NCHN);
    for _ in 0..NCHN {
      adc.push(Vec::<u16>::new());
    }
    Self {
      data_type    : DataType::Unknown,
      status       : EventStatus::Unknown,
      header       : RBEventHeader::new(),
      adc          : adc,
      hits         : Vec::<TofHit>::new(),
    }
  }

  #[deprecated(since="0.11", note="check seems meaningnless")] 
  pub fn trace_check(&self) -> bool {
    let mut check  = true;
    let mut nchan  = 0usize;
    let mut failed = true;
    for ch in self.header.get_channels() {
      if self.adc[ch as usize].len() != NWORDS {
        check = false;
      }
      for k in &self.adc[ch as usize] {
        if *k != u16::MAX {
          // just check that not all bins are 
          // u16::MAX. They get set to that
          // value in case of an error
          // also if that happens to any of 
          // the channels, throw the whole 
          // event away.
          failed = false;
        }
      }
      nchan += 1;
    }
    // for the calibration we want to have all 
    // channels!
    check && nchan == NCHN && !failed
  }

  pub fn has_any_mangling_flag(&self) -> bool {
    match self.status {
      EventStatus::ChnSyncErrors 
      | EventStatus::CellSyncErrors
      | EventStatus::CellAndChnSyncErrors => {
        return true;
      }
      _ => {
        return false;
      }
    }
  }

  /// Check if we have all the channel data even as 
  /// indicated by the header
  pub fn self_check(&self) -> Result<(),AnalysisError>  {
    let mut pass = false;
    for ch in self.header.get_channels() {
      if self.adc[ch as usize].len() == 0 {
        error!("RB {} expects ch {} but it is empty!", self.header.rb_id, ch + 1);
        println!("{}", self.header);
        pass = false;
      }
    }
    if !pass {
      return Err(AnalysisError::MissingChannel);
    }
    Ok(())
  }

  /// Deconstruct the RBEvent to form RBWaveforms
  pub fn get_rbwaveforms(&self) -> Vec<RBWaveform> {
    // FIXME - fix it, this drives me crazy
    let mut waveforms   = Vec::<RBWaveform>::new();
    // at max, we can have 4 waveform packets
    let active_channels = self.header.get_channels();
    let pid             = self.header.get_rbpaddleid();
    if active_channels.contains(&0) || active_channels.contains(&1) {
      let paddle_id  = pid.get_paddle_id(1);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // then b is channel 1 (or 0)
        wf.adc_b   = self.adc[0].clone();
        wf.adc_a   = self.adc[1].clone();
        wf.rb_channel_b = 0;
        wf.rb_channel_a = 1;
      } else {
        wf.adc_a   = self.adc[0].clone();
        wf.adc_b   = self.adc[1].clone();
        wf.rb_channel_b = 1;
        wf.rb_channel_a = 0;
      }
      waveforms.push(wf);
    }
    if active_channels.contains(&2) || active_channels.contains(&3) {
      let paddle_id  = pid.get_paddle_id(3);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // channel order flipped!
        wf.adc_b   = self.adc[2].clone();
        wf.adc_a   = self.adc[3].clone();
        wf.rb_channel_b = 2;
        wf.rb_channel_a = 3;
      } else {
        wf.adc_a   = self.adc[2].clone();
        wf.adc_b   = self.adc[3].clone();
        wf.rb_channel_b = 3;
        wf.rb_channel_a = 2;
      }
      waveforms.push(wf);
    }
    if active_channels.contains(&4) || active_channels.contains(&5) {
      let paddle_id  = pid.get_paddle_id(5);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // then b is channel 1 (or 0)
        wf.adc_b   = self.adc[4].clone();
        wf.adc_a   = self.adc[5].clone();
        wf.rb_channel_b = 4;
        wf.rb_channel_a = 5;
      } else {
        wf.adc_a   = self.adc[4].clone();
        wf.adc_b   = self.adc[5].clone();
        wf.rb_channel_b = 5;
        wf.rb_channel_a = 4;
      }
      waveforms.push(wf);
    }
    if active_channels.contains(&6) || active_channels.contains(&7) {
      let paddle_id  = pid.get_paddle_id(7);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // then b is channel 1 (or 0)
        wf.adc_b   = self.adc[6].clone();
        wf.adc_a   = self.adc[7].clone();
        wf.rb_channel_b = 6;
        wf.rb_channel_a = 7;
      } else {
        wf.adc_a   = self.adc[6].clone();
        wf.adc_b   = self.adc[7].clone();
        wf.rb_channel_b = 6;
        wf.rb_channel_a = 7;
      }
      waveforms.push(wf);
    }
    waveforms
  }

  /// Get the datatype from a bytestream when we know
  /// that it is an RBEvent
  ///
  /// The data type is encoded in byte 3
  pub fn extract_datatype(stream : &Vec<u8>) -> Result<DataType, SerializationError> {
    if stream.len() < 3 {
      return Err(SerializationError::StreamTooShort);
    }
    // TODO This might panic! Is it ok?
    Ok(DataType::try_from(stream[2]).unwrap_or(DataType::Unknown))
  }
  
  /// decode the len field in the in memroy represention of 
  /// RBEventMemoryView
  pub fn get_channel_packet_len(stream : &Vec<u8>, pos : usize) -> Result<(usize, Vec::<u8>), SerializationError> {
    // len is at position 4 
    // roi is at postion 6
    if stream.len() < 8 {
      return Err(SerializationError::StreamTooShort);
    }
    let mut _pos = pos + 4;
    let packet_len = parse_u16(stream, &mut _pos) as usize * 2; // len is in 2byte words
    if packet_len < 44 {
      // There is only header data 
      error!("Event fragment - no channel data!");
      return Ok((packet_len.into(), Vec::<u8>::new()));
    }
    let nwords     = parse_u16(stream, &mut _pos) as usize + 1; // roi is max bin (first is 0)
    debug!("Got packet len of {} bytes, roi of {}", packet_len, nwords);
    let channel_packet_start = pos + 36;
    let nchan_data = packet_len - 44;
    if stream.len() < channel_packet_start + nchan_data {
      error!("We claim there should be channel data, but the event is too short!");
      return Err(SerializationError::StreamTooShort)
    }

    let mut nchan = 0usize;
    //println!("========================================");
    //println!("{} {} {}", nchan, nwords, nchan_data);
    //println!("========================================");
    while nchan * (2*nwords + 6) < nchan_data {
      nchan += 1;
    }
    if nchan * (2*nwords + 6) != nchan_data {
      error!("NCHAN consistency check failed! nchan {} , nwords {}, packet_len {}", nchan, nwords, packet_len);
    }
    let mut ch_ids = Vec::<u8>::new();
    _pos = channel_packet_start;
    for _ in 0..nchan {
      ch_ids.push(parse_u16(stream, &mut _pos) as u8);
      _pos += (nwords*2) as usize;
      _pos += 4; // trailer
    }
    debug!("Got channel ids {:?}", ch_ids);
    Ok((nchan_data.into(), ch_ids))
  }

  /// Get the event id from a RBEvent represented by bytestream
  /// without decoding the whole event
  ///
  /// This should be faster than decoding the whole event.
  pub fn extract_eventid(stream : &Vec<u8>) -> Result<u32, SerializationError> {
    if stream.len() < 30 {
      return Err(SerializationError::StreamTooShort);
    }
    // event header starts at position 7
    // in the header, it is as positon 3
    let event_id = parse_u32(stream, &mut 10);
    Ok(event_id)
  }

  pub fn get_ndatachan(&self) -> usize {
    self.adc.len()
  }

  pub fn get_channel_by_id(&self, ch : usize) -> Result<&Vec::<u16>, UserError> {
    if ch >= 9 {
      error!("channel_by_id expects numbers from 0-8!");
      return Err(UserError::IneligibleChannelLabel)
    }
    return Ok(&self.adc[ch]);
  }

  pub fn get_channel_by_label(&self, ch : u8) -> Result<&Vec::<u16>, UserError>  {
    if ch == 0 || ch > 9 {
      error!("channel_by_label expects numbers from 1-9!");
      return Err(UserError::IneligibleChannelLabel)
    }
    Ok(&self.adc[ch as usize -1])
  }

  /// Similar to the "official" from_bytestream, this will get the 
  /// event from a bytestream, omitting the waveforms. This will allow
  /// for a faster readout in case waveforms are not needed.
  pub fn from_bytestream_nowaveforms(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut event = Self::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    event.data_type = DataType::try_from(parse_u8(stream, pos)).unwrap_or(DataType::Unknown);
    event.status    = EventStatus::try_from(parse_u8(stream, pos)).unwrap_or(EventStatus::Unknown);
    //let nchan_data  = parse_u8(stream, pos);
    let n_hits      = parse_u8(stream, pos);
    event.header    = RBEventHeader::from_bytestream(stream, pos)?;
    //let ch_ids      = event.header.get_active_data_channels();
    let stream_len  = stream.len();
    if event.header.is_event_fragment() {
      debug!("Fragmented event {} found!", event.header.event_id);
      let tail_pos = seek_marker(stream, Self::TAIL, *pos)?;
      * pos = tail_pos + 2 as usize;
      // the event fragment won't have channel data, so 
      // let's move on to the next TAIL marker:ta
      return Ok(event);
    }
    if event.header.drs_lost_trigger() {
      debug!("Event {} has lost trigger!", event.header.event_id);
      let tail_pos = seek_marker(stream, Self::TAIL, *pos)?;
      * pos = tail_pos + 2 as usize;
      return Ok(event);
    }
    let mut decoded_ch = Vec::<u8>::new();
    // here is the only change to from_bytestream. 
    // We are simply skipping the waveforms, but 
    // still read the hits
    for ch in event.header.get_channels().iter() {
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for event {} ch {} seems corrupt! We want to get channels {:?}, but have decoded only {:?}, because the stream ends {} bytes too early!",event.header.event_id, ch, event.header.get_channels(), decoded_ch, *pos + 2*NWORDS - stream_len);
        let tail_pos = seek_marker(stream, Self::TAIL, *pos)?;
        * pos = tail_pos + 2 as usize;
        return Err(SerializationError::WrongByteSize {})
      }
      decoded_ch.push(*ch);
      // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
      *pos += 2*NWORDS;
    }
    for _ in 0..n_hits {
      match TofHit::from_bytestream(stream, pos) {
        Err(err) => {
          error!("Can't read TofHit! Err {err}");
          let mut h = TofHit::new();
          h.valid = false;
          event.hits.push(h);
        },
        Ok(h) => {
          event.hits.push(h);
        }
      }
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("After parsing the event, we found an invalid tail signature {}", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(event)
  }
}

impl TofPackable for RBEvent {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::RBEvent;
}

impl Serialization for RBEvent {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut event = Self::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    event.data_type = DataType::try_from(parse_u8(stream, pos)).unwrap_or(DataType::Unknown);
    event.status    = EventStatus::try_from(parse_u8(stream, pos)).unwrap_or(EventStatus::Unknown);
    //let nchan_data  = parse_u8(stream, pos);
    let n_hits      = parse_u8(stream, pos);
    event.header    = RBEventHeader::from_bytestream(stream, pos)?;
    //let ch_ids      = event.header.get_active_data_channels();
    let stream_len  = stream.len();
    if event.header.is_event_fragment() {
      debug!("Fragmented event {} found!", event.header.event_id);
      let tail_pos = seek_marker(stream, Self::TAIL, *pos)?;
      * pos = tail_pos + 2 as usize;
      // the event fragment won't have channel data, so 
      // let's move on to the next TAIL marker:ta
      return Ok(event);
    }
    if event.header.drs_lost_trigger() {
      debug!("Event {} has lost trigger!", event.header.event_id);
      let tail_pos = seek_marker(stream, Self::TAIL, *pos)?;
      * pos = tail_pos + 2 as usize;
      return Ok(event);
    }
    let mut decoded_ch = Vec::<u8>::new();
    for ch in event.header.get_channels().iter() {
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for event {} ch {} seems corrupt! We want to get channels {:?}, but have decoded only {:?}, because the stream ends {} bytes too early!",event.header.event_id, ch, event.header.get_channels(), decoded_ch, *pos + 2*NWORDS - stream_len);
        let tail_pos = seek_marker(stream, Self::TAIL, *pos)?;
        * pos = tail_pos + 2 as usize;
        return Err(SerializationError::WrongByteSize {})
      }
      decoded_ch.push(*ch);
      // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
      let data = &stream[*pos..*pos+2*NWORDS];
      //event.adc[k as usize] = u8_to_u16(data);
      event.adc[*ch as usize] = u8_to_u16(data);
      *pos += 2*NWORDS;
    }
    for _ in 0..n_hits {
      match TofHit::from_bytestream(stream, pos) {
        Err(err) => {
          error!("Can't read TofHit! Err {err}");
          let mut h = TofHit::new();
          h.valid = false;
          event.hits.push(h);
        },
        Ok(h) => {
          event.hits.push(h);
        }
      }
    }
    let tail = parse_u16(stream, pos);
    //println!("{:?}", &stream[*pos-10..*pos+2]);
    //println!("{} {}", pos, stream.len());
    if tail != Self::TAIL {
      error!("After parsing the event, we found an invalid tail signature {}", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(event)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(18530);
    //let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.data_type as u8);
    stream.push(self.status as u8);
    //let nchan_data  = self.adc.len() as u8;
    //stream.push(nchan_data);
    let n_hits      = self.hits.len() as u8;
    stream.push(n_hits);
    stream.extend_from_slice(&self.header.to_bytestream());
    // for an empty channel, we will add an empty vector
    let add_channels = !self.header.is_event_fragment() & !self.header.drs_lost_trigger();
    if add_channels {
      for n in 0..NCHN {
        for k in 0..NWORDS {
          if self.adc[n].len() == 0 {
            continue;
          }
          stream.extend_from_slice(&self.adc[n][k].to_le_bytes());  
        }
      }
      // this is way slower
      //for channel_adc in self.adc.iter() {
      //  stream.extend_from_slice(&u16_to_u8(&channel_adc)); 
      //}
    }
    //if self.ch9_adc.len() > 0 {
    //  stream.extend_from_slice(&u16_to_u8(&self.ch9_adc));
    //}
    for h in self.hits.iter() {
      stream.extend_from_slice(&h.to_bytestream());
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl Default for RBEvent {

  fn default () -> Self {
    Self::new()
  }
}

impl fmt::Display for RBEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut adc = Vec::<usize>::new();
    for k in 0..self.adc.len() -1 {
      adc.push(self.adc[k].len());
    }
    let mut ch9_str = String::from("[");
    for k in self.adc[8].iter().take(5) {
      ch9_str += &k.to_string();
      ch9_str += ","
    }
    ch9_str += " .. :";
    ch9_str += &self.adc[8].len().to_string();
    ch9_str += "]";
    let mut ch_field = String::from("[\n");
    for (ch, vals) in self.adc.iter().enumerate() {
      if ch == 8 {
        continue;
      }
      let label = (ch + 1).to_string();
      ch_field += "  [ch ";
      ch_field += &ch.to_string();
      ch_field += "('";
      ch_field += &label;
      ch_field += "') ";
      for n in vals.iter().take(5) {
        ch_field += &n.to_string();
        ch_field += ",";
      }
      ch_field += "..:";
      ch_field += &vals.len().to_string();
      ch_field += "]\n";
    }
    ch_field += "]\n";
    write!(f, "<RBEvent 
  data type     : {},
  event status  : {},
  {}
  .. .. 
  has ch9       : {},
    -> ch9      : {},
  data channels : 
    -> {},
  n hits        : {},
.. .. .. .. .. .. .. .. >",
    self.data_type,
    self.status,
    self.header,
    self.header.has_ch9(),
    ch9_str,
    ch_field,
    self.hits.len())
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEvent {
    
  fn from_random() -> Self {
    let mut event   = RBEvent::new();
    let header      = RBEventHeader::from_random();
    let mut rng     = rand::rng();
    event.data_type = DataType::from_random(); 
    event.status    = EventStatus::from_random();
    event.header    = header;
    // set a good status byte. RBEvents from 
    // random will always be good
    // status_byte is tested in RBEventHeader test
    // and here we want to make sure channel data 
    // gets serialized
    // status byte of 0 means it is good
    event.header.status_byte = 0;
    //if !event.header.event_fragment && !event.header.lost_trigger {
    for ch in event.header.get_channels().iter() {
      let random_numbers: Vec<u16> = (0..NWORDS).map(|_| rng.random()).collect();
      event.adc[*ch as usize] = random_numbers;
    }
    //}
    event
  }
}


//impl From<&TofPacket> for RBEvent {
//  fn from(pk : &TofPacket) -> Self {
//    match pk.packet_type {
//      //PacketType::RBEventMemoryView => {
//      //  match RBEvent::extract_from_rbeventmemoryview(&pk.payload, &mut 0) {
//      //    Ok(event) => {
//      //      return event;
//      //    }
//      //    Err(err) => { 
//      //      error!("Can not get RBEvent from RBEventMemoryView! Error {err}!");
//      //      error!("Returning empty event!");
//      //      return RBEvent::new();
//      //    }
//      //  }
//      //},
//      TofPacketType::RBEvent => {
//        match RBEvent::from_bytestream(&pk.payload, &mut 0) {
//          Ok(event) => {
//            return event;
//          }
//          Err(err) => { 
//            error!("Can not decode RBEvent - will return empty event! {err}");
//            return RBEvent::new();
//          }
//        }
//      },
//      TofPacketType::RBEventMemoryView => {
//        let mut streamer = RBEventMemoryStreamer::new();
//        streamer.add(&pk.payload, pk.payload.len());
//        match streamer.get_event_at_pos_unchecked(None) {
//          None => {
//            return RBEvent::new();
//          },
//          Some(ev) => {
//            return ev;
//          }
//        }
//      },
//
//      _ => {
//        error!("Can not deal with {}! Returning empty event", pk);
//        return RBEvent::new();
//      }
//    }
//  }
//}

//-----------------------------------------------------------
  
#[cfg(feature="pybindings")]
#[pymethods]
impl RBEvent {

  #[getter]
  fn get_status(&self) -> EventStatus {
    self.status
  }

  // FIXME - use PyReadonlyArray? Needs test
  /// Get adc values directly from the RBEvent with zero copy
  ///
  /// The channel has to go from 1-9
  fn get_waveform<'_py>(&self, py: Python<'_py>, channel : usize) -> PyResult<Bound<'_py, PyArray1<u16>>> {  
    if channel < 1 || channel > 9 {
      return Err(PyValueError::new_err("Channel must be > 0 and < 9"));
    }
    let data = &self.adc[channel - 1];
    let py_array = data.to_pyarray(py);
    Ok(py_array)
  }
  
  fn get_waveform_slow<'_py>(&self, py: Python<'_py>, channel : usize) -> PyResult<Bound<'_py, PyArray1<u16>>> {  
    let wf  = self.get_channel_by_id(channel).unwrap().clone();
    let arr = PyArray1::<u16>::from_vec(py, wf);
    Ok(arr)
  }
  
  
  #[getter]
  fn get_hits(&self) -> Vec<TofHit> {
    self.hits.clone()
  }
 
  // FIXME - no clear if a new object is created here 
  #[getter]
  fn get_header<'_py>(&self, py : Python<'_py>) -> PyResult<Bound<'_py , RBEventHeader>> {
    Bound::<'_py, RBEventHeader>::new(py, self.header)
  }
  
  #[getter]
  fn waveforms(&self) -> Vec<RBWaveform> {
    self.get_rbwaveforms().clone()
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(RBEvent);

//-----------------------------------------------------------

//FIXME - test needs some review
#[test]
fn serialization_rbevent() {
  for _ in 0..100 {
    let event  = RBEvent::from_random();
    let stream = event.to_bytestream();
    println!("[test rbevent] stream.len()   {:?}", stream.len());
    let test   = RBEvent::from_bytestream(&stream, &mut 0).unwrap();
    println!("[test rbevent] event frag   {:?}", event.header.is_event_fragment());
    println!("[test rbevent] lost trig    {:?}", event.header.drs_lost_trigger());
    println!("[test rbevent] event frag   {:?}", test.header.is_event_fragment());
    println!("[test rbevent] lost trig    {:?}", test.header.drs_lost_trigger());
    assert_eq!(event.header, test.header);
    assert_eq!(event.header.get_nchan(), test.header.get_nchan());
    assert_eq!(event.header.get_channels(), test.header.get_channels());
    assert_eq!(event.data_type, test.data_type);
    assert_eq!(event.status, test.status);
    assert_eq!(event.adc.len(), test.adc.len());
    assert_eq!(event.hits.len(), test.hits.len());
    println!("[test rbevent] get_channels() {:?}", event.header.get_channels());
    assert_eq!(event.adc[0].len(), test.adc[0].len());
    assert_eq!(event.adc[1].len(), test.adc[1].len());
    assert_eq!(event.adc[2].len(), test.adc[2].len());
    assert_eq!(event.adc[3].len(), test.adc[3].len());
    assert_eq!(event.adc[4].len(), test.adc[4].len());
    assert_eq!(event.adc[5].len(), test.adc[5].len());
    assert_eq!(event.adc[6].len(), test.adc[6].len());
    assert_eq!(event.adc[7].len(), test.adc[7].len());
    assert_eq!(event.adc[8].len(), test.adc[8].len());
    assert_eq!(event.adc[0], test.adc[0]);
    assert_eq!(event.adc[1], test.adc[1]);
    assert_eq!(event.adc[2], test.adc[2]);
    assert_eq!(event.adc[3], test.adc[3]);
    assert_eq!(event.adc[4], test.adc[4]);
    assert_eq!(event.adc[5], test.adc[5]);
    assert_eq!(event.adc[6], test.adc[6]);
    assert_eq!(event.adc[7], test.adc[7]);
    assert_eq!(event.adc[8], test.adc[8]);
    //for ch in (event.header.get_channels().iter()){
    //  assert_eq!(event.adc[*ch as usize], test.adc[*ch as usize]);
    //}
    //assert_eq!(event, test);

    //if head.header.event_fragment == test.header.event_fragment {
    //  println!("Event fragment found, no channel data available!");
    //} else {
    //  assert_eq!(head, test);
    //}
  }
}

#[test]
fn pack_rbevent() {
  for _ in 0..100 {
    let event  = RBEvent::from_random();
    let test : RBEvent = event.pack().unpack().unwrap();
    assert_eq!(event, test);
  }
}

