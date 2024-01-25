#include <filesystem>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "serialization.h"
#include "parsers.h"
#include "logging.hpp"
#include "io.hpp"

namespace fs = std::filesystem;

// has to be larger than max packet size
const usize CHUNK_SIZE = 20000;


/***************************************************/

Vec<RBEventHeader> get_rbeventheaders(const String &filename, bool is_headers) {
  spdlog::cfg::load_env_levels();
  u64 n_good = 0;  
  u64 n_bad  = 0; 
  Vec<RBEventHeader> headers;
  bytestream stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  log_info("Read " << stream.size() << " bytes from " << filename);
  log_info("For 8+1 channels and RB compression level 0, this mean a max number of events of " << stream.size()/18530.0);
  while (!has_ended) {
    RBEventHeader header;
    if (is_headers) {
      header = RBEventHeader::from_bytestream(stream, pos);
    } else {
      log_error("Can not deal with this!");
      //header = RBEventHeader::extract_from_rbbinarydump(stream, pos);
    }
    //header.broken ? n_bad++ : n_good++ ;
    headers.push_back(header);
    pos -= 2;
    pos = search_for_2byte_marker(stream, 0xAA, has_ended, pos);
    //if (header.broken) {
    //  std::cout << pos << std::endl;
    //  std::cout << (u32)header.channel_mask << std::endl;
    //}
  }
  log_info("Retrieved " << n_good << " good headers, but " << n_bad << " of which we had to set the `broken` flag");
  return headers;
}

/***************************************************/

Vec<u32> get_event_ids_from_raw_stream(const Vec<u8> &bytestream, u64 &pos) {
  Vec<u32> event_ids;
  u32 event_id = 0;
  // first, we need to find the first header in the 
  // stream starting from the given position
  bool has_ended = false;
  while (!has_ended) { 
    pos = search_for_2byte_marker(bytestream, 0xAA, has_ended, pos);  
    pos += 22;
    event_id = Gaps::u32_from_le_bytes(bytestream, pos);
    event_ids.push_back(event_id);
    pos += 18530 - 22 - 4;
  }
  return event_ids; 
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos) {
  Vec<TofPacket> packets;
  u64 pos  = start_pos;
  // just make sure in the beginning they
  // are not the same
  u64 last_pos = start_pos += 1;
  TofPacket packet;
  u64 n_packets = 0;
  while (true) {
    packet = TofPacket::from_bytestream(bytestream, pos);
    if (pos != last_pos) {
      packets.push_back(packet);
      n_packets += 1;
    } else {
      break;
    }
    last_pos = pos;
  }
  log_info("Read out " << n_packets << " packets from bytestream!");
  return packets;
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const String filename) {
  spdlog::cfg::load_env_levels();
  if (!fs::exists(filename)) {
    log_fatal("Can't open " << filename << " since it does not exist!");
  }

  auto stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  if (has_ended) {
    log_error("The stream ended before we found any header marker!");
  } else {
    log_debug("Found the first header at pos " << pos);
  }
  log_debug("Read " << stream.size() << " bytes from " << filename);
  return get_tofpackets(stream, pos);
}

/***************************************************/

Vec<RBEventMemoryView> get_rbeventmemoryviews(const String &filename, bool omit_duplicates ) {
  spdlog::cfg::load_env_levels();
  auto stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  log_debug("Read " << stream.size() << " bytes from " <<  filename);
  return get_rbeventmemoryviews(stream, pos, omit_duplicates);
}

/***************************************************/

Vec<RBEventMemoryView> get_rbeventmemoryviews(const Vec<u8> &bytestream,
                                              u64 start_pos,
                                              bool omit_duplicates) {
  u64 nevents_in_stream = (float)bytestream.size()/RBEventMemoryView::SIZE;
  log_info("There might be at max " << nevents_in_stream << " events in the stream");
  if (omit_duplicates) {
    log_warn("Will try to elimiinate duplicate events. This might come at a performance cost!");
  }
  Vec<u32> eventid_registry = Vec<u32>();
  usize n_duplicates        = 0;

  Vec<RBEventMemoryView> events; 
  RBEventMemoryView event;
  usize pos              = start_pos;
  bool has_ended         = false;
  usize n_events_decoded = 0;
  usize corrupt_events   = 0;
  while (n_events_decoded < nevents_in_stream + 1) { 
    // where are assuming that there is 
    // less than one event of garbaget
    // at the beginning of the stream
    pos = search_for_2byte_marker(bytestream,
                                  0xaa,
                                  has_ended,
                                  pos,
                                  pos+RBEventMemoryView::SIZE);
    if ((has_ended) || (pos + RBEventMemoryView::SIZE > bytestream.size())) {
      break;
    } 
    event = RBEventMemoryView::from_bytestream(bytestream,
                                               pos);
    if (event.tail != 0x5555) {
      corrupt_events++;
      pos += 2; // skip header
      continue;
    }
    if (omit_duplicates) {
      auto it = std::find(eventid_registry.begin(),
                          eventid_registry.end(),
                          event.event_ctr);
      if (it != eventid_registry.end()) {
        // we have seen this before
        n_duplicates += 1;
        continue;
      } else {
        eventid_registry.push_back(event.event_ctr);
      }
    }
    events.push_back(event);
    n_events_decoded++;
  }
  log_info("Retrieved " << n_events_decoded << " events from stream!");
  if (n_duplicates > 0) {
    log_warn("We have seen " << n_duplicates << " duplicate events!");
  }
  log_info(" " << corrupt_events << " times a header with no corresponding footer was found. This does not necessarily mean there is a problem, instead it could also be padding bytes introduced due to wrapper packages.");
  return events;
  //u64 pos  = start_pos;
  //Vec<RBEventMemoryView> events;
  //// just make sure in the beginning they
  //// are not the same
  //u64 last_pos = start_pos += 1;
  //RBEventMemoryView event;
  //usize nevents = 0;
  //usize nbytes_read = 0;
  //if (stream.size() < RBEventMemoryView::SIZE) {
  //  log_error("Stream of {} bytes is shorter than a single event ({} bytes)!", 
  //                 stream.size(), RBEventMemoryView::SIZE);
  //  return events;
  //}
  //while (true) {
  //  //if (nevents == 100) break;
  //  if (nbytes_read + RBEventMemoryView::SIZE > stream.size()) {
  //    break;
  //  }
  //  last_pos = pos;
  //  event = event.from_bytestream(stream, pos);
  //  nevents += 1;
  //  nbytes_read += RBEventMemoryView::SIZE;
  //  if (pos != last_pos) {
  //    events.push_back(event);
  //  } else {
  //    break;
  //  }
  //}
  //log_info("Retried {} RBEventMemoryViews from file!", nevents);
  //return events;
}

/***************************************************/

Vec<TofEvent> unpack_tofevents_from_tofpackets(const Vec<u8> &bytestream, u64 start_pos) {
  Vec<TofEvent> events = Vec<TofEvent>();
  u64 pos  = start_pos;
  // just make sure in the beginning they
  // are not the same
  u64 last_pos = start_pos += 1;
  TofPacket packet;
  TofEvent event;
  while (true) {
    last_pos = pos;
    packet = TofPacket::from_bytestream(bytestream, pos);
    //if (n_packets == 100) {break;}
    if (pos != last_pos) {
      if (packet.packet_type == PacketType::TofEvent) {
        //log_info("Got packet with payload {}", packet.payload.size());  
        event = TofEvent::from_tofpacket(packet);
        events.push_back(event);
      }
      //log_info("pos: {}", pos);
      //packets.push_back(packet);
      //n_packets += 1;
    } else {
      break;
    }
  }
  log_info("Read " << events.size() << " TofEvents!");
  return events;
}

/***************************************************/

Vec<TofEvent> unpack_tofevents_from_tofpackets(const String filename) {
  Vec<TofEvent> events = Vec<TofEvent>();
  auto stream = get_bytestream_from_file(filename); 
  log_debug("Read " << stream.size() << " bytes from " <<  filename);
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream, 0xAA, has_ended );
  if (has_ended) {
    log_error("Opened file " << filename << " but no start marker " << TofPacket::HEAD << " could be found indicating that this file is no good!");
    return events;
  }
  return unpack_tofevents_from_tofpackets(stream, pos);
}

/***************************************************/


Vec<u8> read_chunk(const String& filename, usize offset) {

  Vec<u8> buffer;
  buffer.reserve(CHUNK_SIZE);
  

  char chunk[CHUNK_SIZE];
  std::ifstream file(filename, std::ios::binary);
  file.seekg(offset);
  while (file.read(chunk, CHUNK_SIZE)) {
    buffer.insert(buffer.end(), chunk, chunk + file.gcount());
  }

  if (file.eof()) {
    // Reached the end of the file
    buffer.insert(buffer.end(), chunk, chunk + file.gcount());
  } else if (!file) {
    // Error occurred while reading the file
    log_error("Failed to read file " << filename);
    buffer.clear();
  }
  return buffer;
}

/***************************************************/

Gaps::TofPacketReader::TofPacketReader(String filename) {
  if (fs::exists(filename)) {
    log_info("Will read packets from " << filename);
    filename_ = filename;
  } else {
    log_error("File " << filename << " does not exist!"); 
    filename_ = "";
    return;
  }
  stream_file_ = std::ifstream(filename_);   
  std::streampos file_s = stream_file_.tellg();
  file_size_ = static_cast<usize>(file_s);
  nchunks_ = file_size_ / CHUNK_SIZE;
  current_pos_ = 0;
  last_packet_ = TofPacket();
}

/***************************************************/

void Gaps::TofPacketReader::process_chunk() {
  auto stream = read_chunk(filename_, current_pos_);
  bool has_ended = false;
  u64 head_pos = search_for_2byte_marker(stream, 0xAA, has_ended);
  if (!(has_ended)) {
    stream = read_chunk(filename_, head_pos);
    last_packet_ = TofPacket::from_bytestream(stream, head_pos);  
  } else {
    last_packet_ = TofPacket();
  }
}

/***************************************************/

String Gaps::TofPacketReader::get_filename() const {
  return filename_;
}

/***************************************************/

TofPacket Gaps::TofPacketReader::get_next_packet() {
    process_chunk();
    return last_packet_;
}

