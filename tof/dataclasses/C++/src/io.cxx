#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "serialization.h"
#include "parsers.h"

#include "io.hpp"

/***************************************************/

Vec<RBEventHeader> get_rbeventheaders(const String &filename, bool is_headers) {
  spdlog::cfg::load_env_levels();
  u64 n_good = 0;  
  u64 n_bad  = 0; 
  Vec<RBEventHeader> headers;
  bytestream stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  spdlog::info("Read {} bytes from {}", stream.size(), filename);
  spdlog::info("For 8+1 channels and RB compression level 0, this mean a max number of events of {}", stream.size()/18530.0);
  while (!has_ended) {
    RBEventHeader header;
    if (is_headers) {
      header = RBEventHeader::from_bytestream(stream, pos);
    } else {
      header = RBEventHeader::extract_from_rbbinarydump(stream, pos);
    }
    header.broken ? n_bad++ : n_good++ ;
    headers.push_back(header);
    pos -= 2;
    pos = search_for_2byte_marker(stream, 0xAA, has_ended, pos);
    //if (header.broken) {
    //  std::cout << pos << std::endl;
    //  std::cout << (u32)header.channel_mask << std::endl;
    //}
  }
  spdlog::info("Retrieved {} good headers, but {} of which we had to set the `broken` flag", n_good, n_bad);
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
  //u64 n_packets = 0;
  while (true) {
    last_pos = pos;
    pos = packet.from_bytestream(bytestream, pos);
    //if (n_packets == 100) {break;}
    if (pos != last_pos) {
      packets.push_back(packet);
      //n_packets += 1;
      //spdlog::info("Have {} packets!", n_packets);
    } else {
      break;
    }
  }
  return packets;
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const String filename) {
  spdlog::cfg::load_env_levels();
  auto stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  spdlog::info("Read {} bytes from {}", stream.size(), filename);
  return get_tofpackets(stream, pos);
}

/***************************************************/

Vec<RBEventMemoryView> get_rbeventmemoryviews(const String &filename, bool omit_duplicates ) {
  spdlog::cfg::load_env_levels();
  auto stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  spdlog::info("Read {} bytes from {}", stream.size(), filename);
  return get_rbeventmemoryviews(stream, pos, omit_duplicates);
}

/***************************************************/

Vec<RBEventMemoryView> get_rbeventmemoryviews(const Vec<u8> &bytestream,
                                              u64 start_pos,
                                              bool omit_duplicates) {
  u64 nevents_in_stream = (float)bytestream.size()/RBEventMemoryView::SIZE;
  spdlog::info("There might be at max {} events in the stream", nevents_in_stream);
  if (omit_duplicates) {
    spdlog::warn("Will try to elimiinate duplicate events. This might come at a performance cost!");
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
    //std::cout << event << std::endl;
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
    pos += RBEventMemoryView::SIZE + 2;
  }
  spdlog::info("Retrieved {} events from stream!", n_events_decoded);
  if (n_duplicates > 0) {
    spdlog::warn("We have seen {} duplicate events!", n_duplicates);
  }
  spdlog::info("{} times a header with no corresponding footer was found. This does not necessarily mean there is a problem, instead it could also be padding bytes introduced due to wrapper packages.", corrupt_events);
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
  //  spdlog::error("Stream of {} bytes is shorter than a single event ({} bytes)!", 
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
  //spdlog::info("Retried {} RBEventMemoryViews from file!", nevents);
  //return events;
}

/***************************************************/

Vec<BlobEvt_t> get_events_from_stream(const Vec<u8> &bytestream,
	       			                  u64 start_pos) {
  u64 nevents_in_stream = (float)bytestream.size()/BLOBEVENTSIZE;
  spdlog::info("There might be at max {} events in the stream", nevents_in_stream);

  Vec<BlobEvt_t> events; 
  BlobEvt_t event;
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
                                  pos+BLOBEVENTSIZE);
    if ((has_ended) || (pos + BLOBEVENTSIZE > bytestream.size())) {
      break;
    } 
    event = decode_blobevent(bytestream,
                             pos);
    if (event.tail != 0x5555) {
      corrupt_events++;
      pos += 2; // skip header
      continue;
    }
    //std::cout << event << std::endl;
    events.push_back(event);
    n_events_decoded++;
    pos += BLOBEVENTSIZE + 2;
  }
  spdlog::info("Retrieved {} events from stream!", n_events_decoded);
  spdlog::info("{} times a header with no corresponding footer was found. This does not necessarily mean there is a problem, instead it could also be padding bytes introduced due to wrapper packages.", corrupt_events);
  return events;
}


