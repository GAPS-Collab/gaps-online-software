#ifndef TELEMETRY_READER_H_INCLUDED
#define TELEMETRY_READER_H_INCLUDED
// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include <iostream>
#include <fstream>

#include "tof_typedefs.h"
#include "telemetry_dataclasses.hpp"
#include "io.hpp"
#ifdef BUILD_CXX_DB
#include "database.h"
#endif 

namespace gondola {

  /// Read serialized TelemetryPackets from an existing file
  ///
  /// Read GAPS binary files ("Berkeley binaries)
  struct TelemetryPacketReader {
    
    TelemetryPacketReader();
    TelemetryPacketReader(std::string pathname);
    TelemetryPacketReader(const TelemetryPacketReader&) = delete;
    
    auto set_path(std::string pathname) -> void;
    
    auto get_filenames() const -> Vec<std::string>; 
    
    auto get_next_packet() -> TelemetryPacket; 

    /// count the packets within the Telemetry fiel 
    auto count_packets() -> u64;

    /// All packets have been read from the file. 
    /// If they should be read again, the reader 
    /// has to be created again
    auto is_exhausted()   const -> bool;

    /// The packet index is a map of TelemetryPacketType -> Number of seen 
    /// packets 
    auto get_packet_index() const -> const HashMap<TelemetryPacketType, u64>&; 
   
    /// In cached mode, the reader will first read all packets from a file and 
    /// then sort them by their timestamp and counter value. 
    /// This comes at a performance cost, but will allow a more precise estimate 
    /// about missing packets.
    ///
    /// To run in cached mode, run ::cache_all_packets before any ::get_next_packet 
    /// calls. 
    auto cache_all_packets() -> void;  
    
    /// Print the package index, converting the TelemetryPacketType to a string for 
    /// implementation independent viewing
    auto print_packet_index() const -> void;  

    /// Restart the reading of packets from the beginning of the underlying buffers 
    auto rewind() -> void;
    
    #ifdef BUILD_CXX_DB 
    /// The map of all paddles. This is needed later on to look up properties 
    /// of the TOF paddles when we are unpacking events 
    TofPaddleMapPtr paddles = nullptr; 
    #endif 

    private:
      /// Indicate if the reader has run out of 
      /// packets 
      bool exhausted_            ;
      
      /// Reader will emit packets from these files,
      /// if one file is exhausted, it moves on to 
      /// the next file automatically
      Vec<std::string> filenames_;       
      /// The index of the file the reader is 
      /// currently reading
      usize file_idx_            ;
      /// depending on the source of the telemetry files, 
      /// there might be duplicates, because we get them
      /// over different streams. 
      /// Suppress these multiple packets
      bool dedup                  ;
      /// Ignore packets that have a gcu time earlier than start_time 
      f64 start_time              ; 
      /// Ignore packets that have a gcu time later than end_time
      f64 end_time                ; 
      /// The current file the reader is actually reading from 
      std::ifstream  stream_file_ ;     
      //file_reader         : BufReader<File>,
      /// Current (byte) position in the file
      usize cursor                ;
      /// Read only packets of type == PacketType
      //pub filter          : TelemetryPacketType,
      /// Number of read packets
      usize n_packs_read_         ;
      /// Number of skipped packets
      usize n_packs_skipped     ;
      /// Skip the first n packets
      usize skip_ahead          ;
      /// Stop reading after n packets
      usize stop_after          ;
      /// Number of encountered duplicates 
      usize n_duplicates        ;
      /// A cache to allow to quench duplicates 
      /// pkt counter -> pkt checksum
      //dedup_cache         : HashMap<u16, VecDeque<u16>>
      auto prime_next_file_() -> void;
      /// Add the packet type to the counters in the index 
      auto register_packet_type_(TelemetryPacketType const &ptyep) -> void;
      /// Remember each packet we have seen and count it 
      HashMap<TelemetryPacketType, u64> packet_index_;
      /// This is only used in cached mode and will contain the whole file 
      /// deserialized as packets 
      Vec<TelemetryPacket> packet_cache_  = {};
      /// An indicator set internally during the caching process 
      bool in_caching_ = false;
  };
}

#endif 
