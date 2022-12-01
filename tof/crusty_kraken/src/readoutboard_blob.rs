
/***********************************/

use crate::constants::{NWORDS, NCHN};

pub fn BLOBEVENTSIZE() -> usize {
  let size = 36 + (NCHN*2) + (NCHN*NWORDS*2) + (NCHN*4) + 8;
  //let return_value : usize;
  //return_value = size as usize;
  return size;
}


/***********************************/

#[derive(Debug, Clone, Copy)]
pub struct BlobData
{
  pub head            : u16, // Head of event marker
  pub status          : u16,
  pub len             : u16,
  pub roi             : u16,
  pub dna             : u64, 
  pub fw_hash         : u16,
  pub id              : u16,   
  pub ch_mask         : u16,
  pub event_ctr       : u32,
  pub dtap0           : u16,
  pub dtap1           : u16,
  pub timestamp       : u64,
  pub ch_head         : [u16; NCHN],
  pub ch_adc          : [[i16; NWORDS];NCHN], 
  pub ch_trail        : [u32; NCHN],
  pub stop_cell       : u16,
  pub crc32           : u32,
  pub tail            : u16, // End of event marker
} 

/***********************************/

impl BlobData {
  pub fn deserialize(&mut self, bytestream : &Vec<u8>, start_pos : usize ) -> usize {
    let mut pos = start_pos;
    let mut raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.head    = u16::from_le_bytes(raw_bytes_2);
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.status  = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.len     = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.roi     = u16::from_le_bytes(raw_bytes_2); 

    let mut raw_bytes_8  = [bytestream[pos + 1],
                            bytestream[pos + 0],
                            bytestream[pos + 3],
                            bytestream[pos + 2],
                            bytestream[pos + 5],
                            bytestream[pos + 4],
                            bytestream[pos + 7],
                            bytestream[pos + 6]];
    pos   += 8;
    self.dna     = u64::from_be_bytes(raw_bytes_8);

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.fw_hash = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.id      = u16::from_le_bytes(raw_bytes_2);    
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.ch_mask = u16::from_le_bytes(raw_bytes_2); 
   
    let mut raw_bytes_4  = [bytestream[pos + 1],
                            bytestream[pos + 0],
                            bytestream[pos + 3],
                            bytestream[pos + 2]];
    pos   += 4; 
    self.event_ctr = u32::from_be_bytes(raw_bytes_4); 


    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap0   = u16::from_le_bytes(raw_bytes_2); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap1   = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_8  = [0,0,bytestream[pos+1],
                    bytestream[pos + 0],
                    bytestream[pos + 3],
                    bytestream[pos + 2],
                    bytestream[pos + 5],
                    bytestream[pos + 4]];
    pos += 6;
    self.timestamp  = u64::from_be_bytes(raw_bytes_8); 
    for n in 0..NCHN {
        raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
        self.ch_head[n] = u16::from_le_bytes(raw_bytes_2);
        pos   += 2;
        for k in 0..NWORDS {
            raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
            self.ch_adc[n][k] = i16::from_le_bytes(raw_bytes_2);
            pos += 2;
        }
        raw_bytes_4  = [bytestream[pos + 1],
                        bytestream[pos + 0],
                        bytestream[pos + 3],
                        bytestream[pos + 2]];
        pos   += 4; 
        self.ch_trail[n] = u32::from_be_bytes(raw_bytes_4); 
    }

    raw_bytes_2  = [bytestream[pos+0],bytestream[pos + 1]];
    pos   += 2;
    self.stop_cell       = u16::from_le_bytes(raw_bytes_2); 
    raw_bytes_4  = [bytestream[pos + 1],
                    bytestream[pos + 0],
                    bytestream[pos + 3],
                    bytestream[pos + 2]];
    pos   += 4; 
    self.crc32   = u32::from_be_bytes(raw_bytes_4); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.tail    = u16::from_le_bytes(raw_bytes_2);  // End of event marker
    return pos;
  }

  pub fn print (&self) {
    println!("======");
    println!("==> HEAD       {} ", self.head);
    println!("==> STATUS     {} ", self.status);
    println!("==> LEN        {} ", self.len);
    println!("==> ROI        {} ", self.roi);
    println!("==> DNA        {} ", self.dna);
    println!("==> FW_HASH    {} ", self.fw_hash);
    println!("==> ID         {} ", self.id);
    println!("==> CH_MASK    {} ", self.ch_mask);
    println!("==> EVT_CTR    {} ", self.event_ctr);
    println!("==> DTAP0      {} ", self.dtap0);
    println!("==> DTAP1      {} ", self.dtap1);
    println!("==> TIMESTAMP  {} ", self.timestamp);
    println!("==> STOP_CELL  {} ", self.stop_cell);
    println!("==> CRC32      {} ", self.crc32);
    println!("==> TAIL       {} ", self.tail);
    println!("======");

  }
}    

/***********************************/

impl Default for BlobData {
    fn default() -> BlobData {
        BlobData {
            head            : 0, // Head of event marker
            status          : 0,
            len             : 0,
            roi             : 0,
            dna             : 0, 
            fw_hash         : 0,
            id              : 0,   
            ch_mask         : 0,
            event_ctr       : 0,
            dtap0           : 0,
            dtap1           : 0,
            timestamp       : 0,
            ch_head         : [0; NCHN],
            ch_adc          : [[0; NWORDS]; NCHN],
            ch_trail        : [0; NCHN],
            stop_cell       : 0,
            crc32           : 0,
            tail            : 0, // End of event marker
        }
    }
}

/***********************************/



