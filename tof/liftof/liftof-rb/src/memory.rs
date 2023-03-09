extern crate memmap;

use std::error::Error;
use std::fs::File;
use std::fmt;

use memmap::{Mmap,
             MmapMut};

use std::ptr;
///  memory locations for the control 
///  registers
///  /dev/uio0 - DRS4 control
///  /dev/uio1 - buffer 1 for blobs
///  /dev/uio2 - buffer 2 for blobs
pub const UIO0 : &'static str = "/dev/uio0";
pub const UIO1 : &'static str = "/dev/uio1";
pub const UIO2 : &'static str = "/dev/uio2";

/// Data buffer related constants
/// The data buffer is /dev/uio1 
/// and /dev/uio2 are internally
/// a single buffer but with 2 halves.
/// 
/// Interestingly, there is a discrepancy 
/// between the dma_reset when it writes
/// 68176064
pub const DATABUF_TOTAL_SIZE : usize = 66524928;
pub const EVENT_SIZE         : usize = 18530; 
//pub const UIO1_MIN_OCCUPANCY : u32 = 68176064;
pub const UIO1_MIN_OCCUPANCY : u32 = 68157440;
pub const UIO2_MIN_OCCUPANCY : u32 = 135266304;

pub const UIO1_MAX_OCCUPANCY : u32 = 117089408;
pub const UIO2_MAX_OCCUPANCY : u32 = 201788800;

/// The size of a 32bit unsigned int in byte
/// (all words in registers are u32)
pub const SIZEOF_U32 : usize = 4;


#[derive(Debug, Copy, Clone)]
pub struct RegisterError {
}

impl fmt::Display for RegisterError {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", "<RegisterError>")
  }
}

///! There are 2 data buffers, commonly 
///  denoted as "A" and "B".
///  A -> /dev/uio1
///  B -> /dev/uio2
#[derive(Debug, Copy, Clone)]
pub enum BlobBuffer {
  A,
  B,
  //Both
}

impl BlobBuffer {
  pub fn invert(&self) -> BlobBuffer {
    match self {
      BlobBuffer::A => {return BlobBuffer::B},
      BlobBuffer::B => {return BlobBuffer::A}
    }
  }
}

/// Get a size which accomodates nevents
///
/// This means if the given size is too small
/// make sure that at least the whole next
/// event "fits in"
pub fn size_in_events(size : usize) -> usize {
  size/EVENT_SIZE
}

/// Allow READ access to the memory registers at /dev/uio**
///
/// Remember we have a 32bit system
///
///
pub fn map_physical_mem_read(addr_space : &str,
                             addr: u32,
                             len: usize) -> Result<Mmap, Box<dyn Error>> {
  let m = unsafe {
    memmap::MmapOptions::new()
      .offset(addr as u64)
      .len(len)
      .map(&File::open(addr_space)?)?
    };
  Ok(m)
}

/// Allow WRITE access to the memory registers at /dev/uio0
/// 
/// Write control registers.
/// Remember we have a 32bit system
///
/// # Arguments
///
/// addr : The memory address (address8) the register
///        is mapped to.
///
///
pub fn map_physical_mem_write(addr_space : &str,
                              addr       : u32,
                              len        : usize)
  -> Result<MmapMut, Box<dyn Error>> {
  let m = unsafe {
    memmap::MmapOptions::new()
      .offset(addr as u64)
      .len(len)
      .map_mut(&File::options()
        .read(true)
        .write(true)
        .open(addr_space)?)?
    };
  Ok(m)
}

///! Get a single value from a 32bit (1 word) register
///  This reads ONLY control registers 
///  (in /dev/uio0)
///  
///  # Arguments:
///
///  * addr : The addr8 of the register 
///           in /dev/uio0
/// 
pub fn read_control_reg(addr : u32) 
  -> Result<u32, RegisterError> 
  where
    u32: std::fmt::LowerHex, {
  
  //let sz = std::mem::size_of::<u32>();
  let m = match map_physical_mem_read(UIO0, addr, SIZEOF_U32) {
    Ok(m) => m,
    Err(err) => {
      let error = RegisterError {};
      warn!("Failed to mmap: Err={:?}", err);
      return Err(error);
    }
  };
  let p = m.as_ptr() as *const u32;
  let value : u32;
  unsafe {
    value = std::ptr::read_volatile(p.offset(0));
  }
  Ok(value)
}

/// 
pub fn write_control_reg(addr       : u32,
                         data       : u32) 
  -> Result<(), RegisterError> 
  where
    u32: std::fmt::LowerHex, {
  
  trace!("Attempting to write {data} at addr {addr}");
  //let sz = std::mem::size_of::<u32>();
  let m = match map_physical_mem_write(UIO0,addr,SIZEOF_U32) {
    Ok(m) => m,
    Err(err) => {
      let error = RegisterError {};
      warn!("[write_control_reg] Failed to mmap: Err={:?}", err);
      return Err(error);
    }
  };
  let p = m.as_ptr() as *mut u32;
  unsafe {
    std::ptr::write_volatile(p.offset(0), data);
  }
  Ok(())
}

/// For debugging. This just prints the 
/// memory at a certain address
#[deprecated(since = "0.1.0", note = "This just prints out bare memory and is only useful for debugging in the very early dev")]
fn dump_mem<T>(addr_space : &str, addr: u32, len: usize)
where
    T: std::fmt::LowerHex,
{
    let sz = std::mem::size_of::<T>();
    let m = match map_physical_mem_read(addr_space,addr, len * sz) {
        Ok(m) => m,
        Err(err) => {
            panic!("Failed to mmap: Err={:?}", err);
        }
    };
    let p = m.as_ptr() as *const T;
    (0..len).for_each(|x| unsafe {
        println!(
            "{:016x}: {:02$x}",
            addr as usize + sz * x,
            std::ptr::read_volatile(p.offset(x as isize)),
            (sz * 2) as usize
        );
    });
}

///  Read one of the data buffers and return a bytestream 
///  from the given address with the length in events.
///  
///  # Arguments
///
///  * which : Select data buffer to read 
///  * size  : in bytes
///
pub fn read_data_buffer(which : &BlobBuffer, 
                        size  : usize)
    -> Result<Vec::<u8>, RegisterError> 
  where
    u32: std::fmt::LowerHex, {

  let addr_space;
  match which {
    BlobBuffer::A => addr_space = UIO1,
    BlobBuffer::B => addr_space = UIO2
  }
  //let blobsize = BlobData::SERIALIZED_SIZE;
  //let vec_size = blobsize*len;
  // FIXME - allocate the vector elsewhere and 
  // pass it by reference
  let mut bytestream = Vec::<u8>::with_capacity(size);
  let m = match map_physical_mem_read(addr_space, 0x0, size) {
  //let mut m = match map_physical_mem_write(addr_space, 0x0, size) {
    Ok(m) => m,
    Err(err) => {
      let error = RegisterError {};
      warn!("Failed to mmap: Err={:?}", err);
      return Err(error);
    }
  };
 
  //ptr::slice_from_raw_parts(raw_pointer, 3) 

  let p = m.as_ptr() as *const u8;
  //let p = m.as_mut_ptr() as *mut u8;
  let slice = ptr::slice_from_raw_parts(p, size);
  unsafe {
    //bytestream  = Vec::<u8>::from_raw_parts(p, size, size);
    bytestream.extend_from_slice(&*slice); 
  }
  Ok(bytestream)
}

