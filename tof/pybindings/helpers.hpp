#include "tof_typedefs.h"
#include "serialization.h"

bytestream wrap_encode_ushort(u16 value, u32 start_pos);

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos);

bytestream wrap_u32_to_le_bytes(u32 value);

bytestream wrap_encode_uint32(u32 value, size_t start_pos);

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos);

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos);

bytestream wrap_encode_uint64(u64 value, size_t start_pos);

