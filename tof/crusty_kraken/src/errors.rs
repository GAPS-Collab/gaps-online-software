///
///
///
///
///



//use std::error::Error;
use std::fmt;

#[cfg(feature = "diagnostics")]
use hdf5;

/*************************************/

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

#[derive(Debug)]
pub struct BlobError {
    //DeserializationError,
    //SerializationError,
    //GenericError
    details : String
}

impl BlobError {
    fn new(msg: &str) -> BlobError {
        BlobError{details: msg.to_string()}
    }
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}


#[cfg(feature = "diagnostics")]
impl From<hdf5::Error> for BlobError {
    fn from(err: hdf5::Error) -> Self {
        BlobError::new(&err.to_string())
    }
}


/*************************************/

