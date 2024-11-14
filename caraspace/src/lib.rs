//! The Caraspace framework provides a stable 'skeleton' 
//! for the different data containers of the project and 
//! unifies the packet approach in a single sub-unit, 
//! called a 'sclerite'. Each sclerite can contain
//! multiple containers
//!

#[macro_use] extern crate log;

pub mod errors;
pub mod parsers;
pub mod serialization;
pub mod prelude;
pub mod frame;
pub mod reader;
pub mod writer;


#[cfg(feature = "random")]
pub trait FromRandom {
  fn from_random() -> Self;
}

