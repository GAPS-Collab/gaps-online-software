
#[derive(Debug)]
pub enum SerializationError {
    //HeaderNotFound,
    TailInvalid,
    HeadInvalid,
    StreamTooShort,
    ValueNotFound,
    EventFragment
}


#[derive(Debug)]
pub enum WaveformError {
    TimeIndexOutOfBounds,
    TimesTooSmall,
    NegativeLowerBound,
    OutOfRangeUpperBound
}
