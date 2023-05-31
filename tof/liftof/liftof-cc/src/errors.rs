///
///
///
///
///



use std::error::Error;
use std::fmt;

#[cfg(feature = "diagnostics")]
use hdf5;

/*************************************/


//#[derive(Debug, Copy, Clone)]
//pub enum EventError {
//    EventIdMismatch
//}
//
//impl fmt::Display for EventError {
//  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//    write!(f,"<EventError>")
//  }
//}
//
//
//impl Error for EventError {
//}

#[derive(Debug)]
pub enum SerializationError {
    //HeaderNotFound,
    TailInvalid,
    StreamTooShort,
    ValueNotFound
}

/*************************************/

#[derive(Debug)]
pub enum WaveformError {
    TimeIndexOutOfBounds,
    TimesTooSmall,
    NegativeLowerBound,
    OutOfRangeUpperBound
}

/*************************************/



/*************************************/

