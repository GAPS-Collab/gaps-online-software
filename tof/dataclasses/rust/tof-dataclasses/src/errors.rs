
#[derive(Debug)]
pub enum SerializationError {
    //HeaderNotFound,
    TailInvalid,
    HeadInvalid,
    StreamTooShort,
    StreamTooLong,
    ValueNotFound,
    EventFragment,
    UnknownPayload
}

#[derive(Debug)]
pub enum DecodingError {
    //HeaderNotFound,
    UnknownType
}

#[derive(Debug)]
pub enum WaveformError {
    TimeIndexOutOfBounds,
    TimesTooSmall,
    NegativeLowerBound,
    OutOfRangeUpperBound
}
