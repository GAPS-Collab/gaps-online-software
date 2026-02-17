#ifndef CARASPACE_H_INCLUDED
#define CARASPACE_H_INCLUDED
#include <iostream>
#include <fstream>
#include <cmath>

#include <memory>
#include "tof_typedefs.h"
#include "packets/tof_packet.h"
#include "telemetry_dataclasses.hpp"
#include "io/telemetry_reader.hpp"
#include "calibration.h"
#include "errors.hpp"
#include "result/result.h"
#ifdef BUILD_CXXDB
#include "database.h"
#endif


namespace gondola {

  /// These are objects which can be stored in a caraspace frame  
  enum class CRFrameObjectType : u8 {
    Unknown           = 0,
    TofPacket         = 10,
    TelemetryPacket   = 20,
  };

  struct CRFrameObject {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
    
    u8 version;
    CRFrameObjectType ftype;
    Vec<u8> payload;
 
    auto to_bytestream() const -> Vec<u8>;

    /// Decode a serializable from a bytestream  
    static auto from_bytestream(Vec<u8> stream, usize &pos) -> CRFrameObject;
     
    /// string representation for printing
    auto to_string() -> std::string;
  };


  struct CRFrame {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
      
    static auto from_bytestream(Vec<u8> stream, usize &pos) -> CRFrame;
    
    std::map<std::string, std::tuple<u64, CRFrameObjectType>> index;
    Vec<u8> bytestorage;
    auto to_string() const -> std::string;
    
    static auto parse_index(Vec<u8> stream, usize &pos) -> std::map<std::string, std::tuple<u64, CRFrameObjectType>>;

    auto put_fobject(CRFrameObject const &fobj, std::string) -> void; 

    /// extract a tofpacket if this frame object is of the correct type
    auto get_tofpacket(std::string name) -> result::Result<TofPacket,gondola::IOError>;
    auto get_telemetrypacket(std::string name) -> Gaps::Telemetry::Packet;
  };

  struct CRReader {
    CRReader();
    CRReader(std::string pathname);
    CRReader(const CRReader&) = delete;
   
    /// Read files from a given pathname, this can be 
    /// a single file as well. If the files are 
    /// telemetry files, we will automatically switch 
    /// to using TelemetryPacketReader under the hood 
    auto set_path(std:: string pathname) -> void;
    /// Walk over the file, and return the next frame
    /// as saved in the file. 
    ///
    /// Advaces all internal position markers. If the 
    /// last frame is reached, an exception will be risen.
    /// After the reader is exhausted, is has to be 
    /// re-initialized
    auto get_next_frame()       -> CRFrame;
    auto get_filenames()  const -> Vec<std::string>;
    /// All packets have been read from the file. 
    /// If they should be read again, the reader 
    /// has to be created again
    auto is_exhausted()   const -> bool;
    /// The number of files this reader has read
    /// from the file
    auto n_packets_read() const -> bool;
    /// get the RBCalibration map
    /// the paramter is the number of RBs we expect in this run
    auto get_rbcalibrations(u8 n_rb) -> RBCalibrationMap;     

    /// Has this been created from telemetry dirctly? 
    auto is_from_telemetry() const -> bool;

    /// An index of seen TelemetryPacketTypes in CRFrames. This might 
    /// be of limited use in case that this reader has been run over 
    /// L0 files, but this feature might get implemented in the future.
    auto get_telemetry_packet_index() const -> const HashMap<TelemetryPacketType, u64> &;

    /// In case we are reading from bin files, we can run the underlying TelemetryPacketReader 
    /// in cached mode, which will allow us to have all packets sorted by time and packet header 
    /// counter in memory. A call to ::cache_telemetry_first right after the instanciation of 
    /// a CRReader will trigger the underlying TelemetryPacketReader to go through the caching 
    /// process. After that, the created frames will be sorted in time and by counter value 
    ///
    /// This has ONLY an effect if we are reading .bin files! 
    auto cache_telemetry_first() -> void;

  private:  
    #ifdef BUILD_CXXDB
    Gaps::TofPaddleMap paddles_          ; 
    #endif 
    bool               exhausted_        ;
    usize              n_packets_read_   ;
    Vec<std::string>   filenames_        ;
    std::ifstream      stream_file_      ;
    usize              fileindex_        ;
    auto               prime_next_file_() -> void;
    /// A "cheat". We can internally rewire CRReader 
    /// to use TelemetryPacketReader instead. This 
    /// will then emit frames with only TelemetryPackets
    /// inside 
    bool             is_from_telemetry_;
    std::unique_ptr<TelemetryPacketReader> telly_reader_;
  };
}
#endif
