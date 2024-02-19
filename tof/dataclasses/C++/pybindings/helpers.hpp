#include <pybind11/numpy.h>

#include "tof_typedefs.h"
#include "serialization.h"
#include "io.hpp"
#include "calibration.h"

#include "packets/monitoring.h"

namespace py = pybind11;

RBCalibration  get_from_califile(const String filename);

Vec<TofPacket> wrap_get_tofpackets_from_file(const String filename, PacketType filter = PacketType::Unknown);

Vec<TofPacket> wrap_get_tofpackets_from_stream(const Vec<u8> &stream, u64 pos, PacketType filter = PacketType::Unknown);

Vec<TofEvent> wrap_unpack_tofevents_from_tofpackets_from_file(const String filename);

Vec<TofEvent> wrap_unpack_tofevents_from_tofpackets_from_stream(const Vec<u8> &stream, u64 pos);

Vec<RBEventMemoryView> wrap_get_rbeventmemoryviews_from_file(const String filename, bool omit_duplicates = false);

Vec<RBEventMemoryView> wrap_get_rbeventmemoryviews_from_stream(const Vec<u8> &stream, u64 pos, bool omit_duplicates = false);

String rbmoni_to_string(const RBMoniData &moni);

String rbeventmemoryview_to_string(const RBEventMemoryView &event);

String tofevent_to_string(const TofEvent &event);

py::array_t<f32> wrap_rbcalibration_voltages_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel);

py::array_t<f32> wrap_rbcalibration_voltages_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel);

py::array_t<f32> wrap_rbcalibration_nanoseconds_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel);

py::array_t<f32> wrap_rbcalibration_nanoseconds_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel);

Vec<py::array_t<f32>> wrap_rbcalibration_voltages_allchan_rbevent(const RBCalibration& calib, const RBEvent& event, bool spike_cleaning = false);

Vec<py::array_t<f32>> wrap_rbcalibration_voltages_allchan_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, bool spike_cleaning = false);

Vec<py::array_t<f32>> wrap_rbcalibration_nanoseconds_allchan_rbevent(const RBCalibration& calib, const RBEvent& event);

Vec<py::array_t<f32>> wrap_rbcalibration_nanoseconds_allchan_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event);

template <typename T>
py::array_t<T> to_nparray(const Vec<T> &vec) {
  // Create a NumPy array from the C++ vector
  py::array_t<T> numpy_array(vec.size(), vec.data());
  return numpy_array;
}

RBCalibration unpack_tp_to_rbcalibration(const TofPacket& tp);

//py::object unpack(const TofPacket& tp);
