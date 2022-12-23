///! Tof dataclasses
///
///
///
///

//pub mod events::blob;
//pub mod events::tof_event;
pub mod events;
pub mod packets;
pub mod errors;
pub mod serialization;


extern crate pretty_env_logger;
#[macro_use] extern crate log;

//pretty_env_logger::init();



pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
