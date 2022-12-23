
#[derive(Debug)]
pub enum SerializationError {
    //HeaderNotFound,
    TailInvalid,
    StreamTooShort,
    ValueNotFound
}

