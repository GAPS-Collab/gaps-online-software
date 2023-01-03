extern crate memmap;

use std::error::Error;
use std::fs::File;

use tof_dataclasses::events::blob::BlobData;


use memmap::{Mmap,
             MmapMut};


#[derive(Debug, Copy, Clone)]
pub struct RegisterError {
}

#[derive(Debug, Copy, Clone)]
pub enum BlobBuffer {
  A,
  B,
  //Both
}



/// Allow READ access to the memory registers at /dev/uio**
///
/// Remember we have a 32bit system
///
///
pub fn map_physical_mem_read(addr_space : &str, addr: u32, len: usize) -> Result<Mmap, Box<dyn Error>> {
  let m = unsafe {
    memmap::MmapOptions::new()
      .offset(addr as u64)
      .len(len)
      .map(&File::open(addr_space)?)?
    };
  Ok(m)
}

/// Allow WRITE access to the memory registers at /dev/uio**
///
/// Remember we have a 32bit system
///
/// # Arguments
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

///! Get a single value from a 1 word register
pub fn read_reg(addr_space : &str, 
                addr       : u32) 
  -> Result<u32, RegisterError> 
  where
    u32: std::fmt::LowerHex, {
  
  let sz = std::mem::size_of::<u32>();
  let m = match map_physical_mem_read(addr_space,addr, sz) {
      Ok(m) => m,
      Err(err) => {
          let error = RegisterError {};
          println!("Failed to mmap: Err={:?}", err);
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
pub fn write_reg(addr_space : &str, 
             addr       : u32,
             data       : u32) 
  -> Result<(), RegisterError> 
  where
    u32: std::fmt::LowerHex, {
  
  let sz = std::mem::size_of::<u32>();
  let m = match map_physical_mem_write(addr_space,addr, sz) {
    Ok(m) => m,
    Err(err) => {
      let error = RegisterError {};
      println!("Failed to mmap: Err={:?}", err);
      return Err(error);
    }
  };
  let mut p = m.as_ptr() as *mut u32;
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

///! Read the data buffers and return a bytestream 
///  from the given address with the length in events.
///
pub fn get_bytestream(addr_space : &str, 
                      addr       : u32,
                      size       : usize) -> Result<Vec::<u8>, RegisterError> 
  where
    u32: std::fmt::LowerHex, {

  //let blobsize = BlobData::SERIALIZED_SIZE;
  //let vec_size = blobsize*len;
  // FIXME - allocate the vector elsewhere and 
  // pass it by reference
  let mut bytestream = Vec::<u8>::with_capacity(size);

  let sz = std::mem::size_of::<u8>();
  let mut failed = false;
  let mut m = match map_physical_mem_read(addr_space, addr, size * sz) {
    Ok(m) => m,
    Err(err) => {
      let error = RegisterError {};
      println!("Failed to mmap: Err={:?}", err);
      failed = true;
      map_physical_mem_read(addr_space, 0x0, 1).unwrap()
      //return Err(error);
    }
  };
  
  let mut n_iter = 0;
  let mut foo;
  //let test = u32::LowerHex(addr);
  while failed {
 
    m = match map_physical_mem_read(addr_space, 0x0, size * sz - n_iter) {
      Ok(m) => {
        failed = false;
        foo = size * sz - n_iter;
        println!("We were successful with a size of {foo}");
        m
      }
      Err(err) => {
        let error = RegisterError {};
        println!("Failed to mmap: Err={:?}", err);
        failed = true;
        n_iter += 1;
        map_physical_mem_read(addr_space, 0x0, 1).unwrap()
        //return Err(error);
      }
    };
  }
  let p = m.as_ptr() as *const u8;
  (0..size).for_each(|x| unsafe {
    let value = std::ptr::read_volatile(p.offset(x as isize));
    bytestream.push(value); // push is free, since we 
                            // allocated the vector in the 
                            // beginning
  });
  Ok(bytestream)
}

