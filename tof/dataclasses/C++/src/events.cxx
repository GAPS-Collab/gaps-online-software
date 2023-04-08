#include "events.h"
#include "parsers.h"
#include "serialization.h"

/*****
 * Get the event id from the first event following an offset in a bytestream
 *
 * Will set the pos variable to the location of the next possible 
 * event
 *
 * @param bytestream : A vector or bytes representing one or more 
 *                     raw readoutboard events ("blob")
 * @param pos        : The position from which to start searching 
 *                     for the next event in the bytestream                    
 */ 
vec_u32 get_event_ids_from_raw_stream(const vec_u8 &bytestream, u64 &pos) {
  //  std::cout << "starting" << std::endl;
  vec_u32 event_ids;

  u32 event_id = 0;
  // first, we need to find the first header in the 
  // stream starting from the given position
  bool has_ended = false;
  while (!has_ended) { 
    pos = search_for_2byte_marker(bytestream, 0xAA, has_ended, pos);  
  //if (has_ended) { 
  //  // FIXME logging?    
  //  return 0;
  //}
  // the event id is 4 bytes following byte 22
    pos += 22;
    event_id = Gaps::u32_from_le_bytes(bytestream, pos);
    event_ids.push_back(event_id);
    pos += 18530 - 22 - 4;
  }
  return event_ids; 
}

