#ifndef CARASPACE_H_INCLUDED
#define CARASPACE_H_INCLUDED
#include <iostream>
#include <fstream>

#include "tof_typedefs.h"
#include "packets/tof_packet.h"
#include "telemetry.hpp"

namespace Gaps {

  enum class CRFrameObjectType : u8 {
    Unknown           = 0,
    TofPacket         = 10,
    TelemetryPacket   = 20,
  };

  struct CRFrameObject {
    static const u16 HEAD = 0xAAAA;
    static const u16 TAIL = 0x5555;
    
    u8 version;
    CRFrameObjectType ftype;
    Vec<u8> payload;
  
    /// Decode a serializable from a bytestream  
    static CRFrameObject from_bytestream(Vec<u8> stream, usize &pos);
     
    /// string representation for printing
    std::string to_string();
  };


  struct CRFrame {
    static const u16 HEAD = 0xAAAA;
    static const u16 TAIL = 0x5555;
      
    //std::map<std::string, usize> get_index
    static CRFrame from_bytestream(Vec<u8> stream, usize &pos);
    
    std::map<std::string, std::tuple<u64, CRFrameObjectType>> index;
    Vec<u8> bytestorage;
    std::string to_string() const;
    
    static std::map<std::string, std::tuple<u64, CRFrameObjectType>> parse_index(Vec<u8> stream, usize &pos);
    
    /// extract a tofpacket if this frame object is of the correct type
    TofPacket get_tofpacket(std::string name);
    Gaps::TelemetryPacket get_telemetrypacket(std::string name);

  //pub fn get<T : CRSerializeable + Frameable>(&self, name : String) -> Result<T, CRSerializationError> {

  };

  struct CRReader {
    CRReader();
    CRReader(std::string filename);
    CRReader(const CRReader&) = delete;
    /// Set a filename where to read packets from. This is a binary file format,
    /// typically ending in ".tof.gaps"
    /// Walk over the file and return the next packet
    void set_filename(std:: string);
    CRFrame get_next_frame();
    std::string get_filename() const;
    /// Return the filename we assigned
    /// All packets have been read from the file. 
    /// If they should be read again, the reader 
    /// has to be created again
    bool      is_exhausted() const;
    /// The number of files this reader has read
    /// from the file
    bool      n_packets_read() const;
  private:  
    bool           exhausted_;
    usize          n_packets_read_;
    std::string    filename_;
    std::ifstream  stream_file_;
  };
}
#endif
