#include "tof_typedefs.h"
#include "serialization.h"
#include "io.hpp"

#include "packets/monitoring.h"

bytestream wrap_encode_ushort(u16 value, u32 start_pos);

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos);

bytestream wrap_u32_to_le_bytes(u32 value);

bytestream wrap_encode_uint32(u32 value, size_t start_pos);

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos);

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos);

bytestream wrap_encode_uint64(u64 value, size_t start_pos);

Vec<TofPacket> wrap_get_tofpacket_from_file(const String filename);

Vec<TofPacket> wrap_get_tofpacket_from_stream(const Vec<u8> &stream, u64 pos);

Vec<RBEventMemoryView> wrap_get_rbeventmemoryview_from_file(const String filename);

Vec<RBEventMemoryView> wrap_get_rbeventmemoryview_from_stream(const Vec<u8> &stream, u64 pos);

String rbmoni_to_string(const RBMoniData &moni);

String rbeventmemoryview_to_string(const RBEventMemoryView &event);

String tofevent_to_string(const TofEvent &event);

String mastertriggerevent_to_string(const MasterTriggerEvent &event);

