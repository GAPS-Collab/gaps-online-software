#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

#include "TofTypeDefs.h"

vec_u32 get_event_ids_from_raw_stream(const vec_u8 &bytestream, u64 &start_pos);
#endif 
