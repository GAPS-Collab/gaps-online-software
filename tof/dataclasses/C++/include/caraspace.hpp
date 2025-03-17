#ifndef CARASPACE_H_INCLUDED
#define CARASPACE_H_INCLUDED
#include <iostream>
#include <fstream>
#include <cmath>

#include "tof_typedefs.h"
#include "packets/tof_packet.h"
#include "telemetry_dataclasses.hpp"
#include "result/result.h"
#include "errors.hpp"

namespace r = result;

namespace Gaps {
  /// Get all files in a certain directory in case it is a directory, for 
  /// a single file just get the file <3 ChatGPT
  std::vector<std::string> list_path_contents_sorted(const std::string& input);
  
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
    auto to_string() const -> std::string;
    
    static std::map<std::string, std::tuple<u64, CRFrameObjectType>> parse_index(Vec<u8> stream, usize &pos);
    
    /// extract a tofpacket if this frame object is of the correct type
    auto get_tofpacket(std::string name) -> r::Result<TofPacket,Gaps::IOError>;
    Gaps::Telemetry::Packet get_telemetrypacket(std::string name);

  //pub fn get<T : CRSerializeable + Frameable>(&self, name : String) -> Result<T, CRSerializationError> {

  };

  struct CRReader {
    CRReader();
    CRReader(std::string pathname);
    void set_path(std:: string pathname);
    CRReader(const CRReader&) = delete;
    CRFrame get_next_frame();
    Vec<std::string> get_filenames() const;
    /// All packets have been read from the file. 
    /// If they should be read again, the reader 
    /// has to be created again
    bool      is_exhausted() const;
    /// The number of files this reader has read
    /// from the file
    bool      n_packets_read() const;

  private:  
    bool             exhausted_        ;
    usize            n_packets_read_   ;
    Vec<std::string> filenames_        ;
    std::ifstream    stream_file_      ;
    usize            fileindex_        ;
    void             prime_next_file_();

  };
}
#endif
