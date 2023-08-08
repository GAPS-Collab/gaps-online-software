#include "tof_typedefs.h"
#include "serialization.h"
#include "io.hpp"
#include "calibration.h"

#include "packets/monitoring.h"

bytestream wrap_encode_ushort(u16 value, u32 start_pos);

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos);

bytestream wrap_u32_to_le_bytes(u32 value);

bytestream wrap_encode_uint32(u32 value, size_t start_pos);

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos);

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos);

bytestream wrap_encode_uint64(u64 value, size_t start_pos);

Vec<TofPacket> wrap_get_tofpackets_from_file(const String filename);

Vec<TofPacket> wrap_get_tofpackets_from_stream(const Vec<u8> &stream, u64 pos);

Vec<RBEventMemoryView> wrap_get_rbeventmemoryviews_from_file(const String filename, bool omit_duplicates = false);

Vec<RBEventMemoryView> wrap_get_rbeventmemoryviews_from_stream(const Vec<u8> &stream, u64 pos, bool omit_duplicates = false);

String rbmoni_to_string(const RBMoniData &moni);

String rbeventmemoryview_to_string(const RBEventMemoryView &event);

String tofevent_to_string(const TofEvent &event);

String mastertriggerevent_to_string(const MasterTriggerEvent &event);
  
Vec<f32> wrap_rbcalibration_voltages_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel);

Vec<f32> wrap_rbcalibration_voltages_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel);

Vec<f32> wrap_rbcalibration_nanoseconds_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel);

Vec<f32> wrap_rbcalibration_nanoseconds_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel);


