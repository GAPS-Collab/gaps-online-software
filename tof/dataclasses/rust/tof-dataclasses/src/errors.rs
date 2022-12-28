
#[derive(Debug)]
pub enum SerializationError {
    //HeaderNotFound,
    TailInvalid,
    StreamTooShort,
    ValueNotFound
}


#[derive(Debug)]
pub enum WaveformError {
    TimeIndexOutOfBounds,
    TimesTooSmall,
    NegativeLowerBound,
    OutOfRangeUpperBound
}
