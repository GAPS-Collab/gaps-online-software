#include "tof_typedefs.h"
#include "serialization.h"
#include "io.hpp"
#include "helpers.hpp"

#include "spdlog/spdlog.h"
#include <pybind11/pybind11.h>

namespace py = pybind11;

/***********************************************/

Vec<TofPacket> wrap_get_tofpackets_from_file(const String filename) {
  return get_tofpackets(filename);
}

/***********************************************/

Vec<TofPacket> wrap_get_tofpackets_from_stream(const Vec<u8> &stream, u64 pos) {
  return get_tofpackets(stream, pos);
}

/***********************************************/

Vec<RBEventMemoryView> wrap_get_rbeventmemoryviews_from_file(const String filename, bool omit_duplicates) {
  return get_rbeventmemoryviews(filename, omit_duplicates);
}

/***********************************************/

Vec<RBEventMemoryView> wrap_get_rbeventmemoryviews_from_stream(const Vec<u8> &stream, u64 pos, bool omit_duplicates) {
  return get_rbeventmemoryviews(stream, pos, omit_duplicates);
}

/***********************************************/

Vec<TofEvent> wrap_unpack_tofevents_from_tofpackets_from_file(const String filename) {
  return unpack_tofevents_from_tofpackets(filename);
}

/***********************************************/

Vec<TofEvent> wrap_unpack_tofevents_from_tofpackets_from_stream(const Vec<u8> &stream, u64 pos) {
  return unpack_tofevents_from_tofpackets(stream, pos);
}

/***********************************************/

String rbmoni_to_string(const RBMoniData &moni) {
  String repr = "<RBMoniData: \n";
  repr += "\t board_id           " + std::to_string(moni.board_id)         + "\n"; 
  repr += "\t rate               " + std::to_string(moni.rate)             + "\n"; 
  repr += "\t tmp_drs            " + std::to_string(moni.tmp_drs)          + "\n"; 
  repr += "\t tmp_clk            " + std::to_string(moni.tmp_clk)          + "\n"; 
  repr += "\t tmp_adc            " + std::to_string(moni.tmp_adc)          + "\n"; 
  repr += "\t tmp_zynq           " + std::to_string(moni.tmp_zynq)         + "\n"; 
  repr += "\t tmp_lis3mdltr      " + std::to_string(moni.tmp_lis3mdltr)    + "\n"; 
  repr += "\t tmp_bm280          " + std::to_string(moni.tmp_bm280)        + "\n"; 
  repr += "\t pressure           " + std::to_string(moni.pressure)         + "\n"; 
  repr += "\t humidity           " + std::to_string(moni.humidity)         + "\n"; 
  repr += "\t mag_x              " + std::to_string(moni.mag_x)            + "\n"; 
  repr += "\t mag_y              " + std::to_string(moni.mag_y)            + "\n"; 
  repr += "\t mag_z              " + std::to_string(moni.mag_z)            + "\n"; 
  repr += "\t mag_tot            " + std::to_string(moni.mag_tot)          + "\n"; 
  repr += "\t drs_dvdd_voltage   " + std::to_string(moni.drs_dvdd_voltage) + "\n"; 
  repr += "\t drs_dvdd_current   " + std::to_string(moni.drs_dvdd_current) + "\n"; 
  repr += "\t drs_dvdd_power     " + std::to_string(moni.drs_dvdd_power)   + "\n"; 
  repr += "\t p3v3_voltage       " + std::to_string(moni.p3v3_voltage)     + "\n"; 
  repr += "\t p3v3_current       " + std::to_string(moni.p3v3_current)     + "\n"; 
  repr += "\t p3v3_power         " + std::to_string(moni.p3v3_power)       + "\n"; 
  repr += "\t zynq_voltage       " + std::to_string(moni.zynq_voltage)     + "\n"; 
  repr += "\t zynq_current       " + std::to_string(moni.zynq_current)     + "\n"; 
  repr += "\t zynq_power         " + std::to_string(moni.zynq_power)       + "\n"; 
  repr += "\t p3v5_voltage       " + std::to_string(moni.p3v5_voltage)     + "\n";  
  repr += "\t p3v5_current       " + std::to_string(moni.p3v5_current)     + "\n"; 
  repr += "\t p3v5_power         " + std::to_string(moni.p3v5_power)       + "\n"; 
  repr += "\t adc_dvdd_voltage   " + std::to_string(moni.adc_dvdd_voltage) + "\n"; 
  repr += "\t adc_dvdd_current   " + std::to_string(moni.adc_dvdd_current) + "\n"; 
  repr += "\t adc_dvdd_power     " + std::to_string(moni.adc_dvdd_power)   + "\n"; 
  repr += "\t adc_avdd_voltage   " + std::to_string(moni.adc_avdd_voltage) + "\n"; 
  repr += "\t adc_avdd_current   " + std::to_string(moni.adc_avdd_current) + "\n"; 
  repr += "\t adc_avdd_power     " + std::to_string(moni.adc_avdd_power)   + "\n"; 
  repr += "\t drs_avdd_voltage   " + std::to_string(moni.drs_avdd_voltage) + "\n"; 
  repr += "\t drs_avdd_current   " + std::to_string(moni.drs_avdd_current) + "\n"; 
  repr += "\t drs_avdd_power     " + std::to_string(moni.drs_avdd_power)   + "\n"; 
  repr += "\t n1v5_voltage       " + std::to_string(moni.n1v5_voltage)     + "\n"; 
  repr += "\t n1v5_current       " + std::to_string(moni.n1v5_current)     + "\n"; 
  repr += "\t n1v5_power         " + std::to_string(moni.n1v5_power)       + "\n"; 
  repr += " >";
  return repr;
}

/***********************************************/

String rbeventmemoryview_to_string(const RBEventMemoryView &event) {
  String repr = "<RBEventMemoryView\n";
  //repr += "\thead "      + std::to_string(event.head )      + "\n" ;
  repr += "\tstatus "    + std::to_string(event.status )    + "\n" ;
  repr += "\tlen "       + std::to_string(event.len )       + "\n" ;
  repr += "\troi "       + std::to_string(event.roi )       + "\n" ;
  repr += "\tdna "       + std::to_string(event.dna )       + "\n" ;
  repr += "\tfw_hash "   + std::to_string(event.fw_hash )   + "\n" ;
  repr += "\tid "        + std::to_string(event.id )        + "\n" ;
  repr += "\tch_mask "   + std::to_string(event.ch_mask )   + "\n" ;
  repr += "\tevent_ctr " + std::to_string(event.event_ctr ) + "\n" ;
  repr += "\tdtap0 "     + std::to_string(event.dtap0 )     + "\n" ;
  repr += "\tdtap1 "     + std::to_string(event.dtap1 )     + "\n" ;
  repr += "\ttimestamp " + std::to_string(event.timestamp ) + "\n" ;
  repr += "\tstop_cell " + std::to_string(event.stop_cell ) + "\n" ;
  repr += "\tcrc32 "     + std::to_string(event.crc32 )      ;
  //repr += "\ttail "      + std::to_string(event.tail)       ;
  repr += ">";
  return repr;
}

/***********************************************/


String rbevent_to_string(const RBEvent &event) {
  String repr = "<RBEvent\n";
  repr += ">";
  return repr;
}

/***********************************************/


py::array_t<f32> wrap_rbcalibration_voltages_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<f32> volts = calib.voltages(event, channel);
  return to_nparray(volts);
}

py::array_t<f32> wrap_rbcalibration_voltages_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<f32> volts = calib.voltages(event, channel);
  return to_nparray(volts);
}

/***********************************************/

py::array_t<f32> wrap_rbcalibration_nanoseconds_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<f32> nanos = calib.nanoseconds(event, channel);
  return to_nparray(nanos);
}

/***********************************************/

py::array_t<f32> wrap_rbcalibration_nanoseconds_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<f32> nanos = calib.nanoseconds(event, channel);
  return to_nparray(nanos);
}

/***********************************************/

Vec<py::array_t<f32>> wrap_rbcalibration_voltages_allchan_rbevent(const RBCalibration& calib, const RBEvent& event, bool spike_cleaning) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<Vec<f32>> volts = calib.voltages(event, spike_cleaning);
  Vec<py::array_t<f32>> arr;
  for (const auto &ch : volts) {
    arr.push_back(to_nparray(ch));
  }
  return arr;
}

Vec<py::array_t<f32>> wrap_rbcalibration_voltages_allchan_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, bool spike_cleaning) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<Vec<f32>> volts = calib.voltages(event, spike_cleaning);
  Vec<py::array_t<f32>> arr;
  for (const auto &ch : volts) {
    arr.push_back(to_nparray(ch));
  }
  return arr;
}

Vec<py::array_t<f32>> wrap_rbcalibration_nanoseconds_allchan_rbevent(const RBCalibration& calib, const RBEvent& event) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<Vec<f32>> nanos = calib.nanoseconds(event);
  Vec<py::array_t<f32>> arr;
  for (const auto &ch : nanos) {
    arr.push_back(to_nparray(ch));
  }
  return arr;
}

Vec<py::array_t<f32>> wrap_rbcalibration_nanoseconds_allchan_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  Vec<Vec<f32>> nanos = calib.nanoseconds(event);
  Vec<py::array_t<f32>> arr;
  for (const auto &ch : nanos) {
    arr.push_back(to_nparray(ch));
  }
  return arr;
}

/***********************************************/

RBCalibration unpack_tp_to_rbcalibration(const TofPacket& tp) {
  if (tp.packet_type != PacketType::RBCalibration) {
    String message = "The TofPacket has the wrong type!" + packet_type_to_string(tp.packet_type);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  u64 pos = 0;
  spdlog::debug("Will call RBCalibration::from_bytestream on packet.payload"); 
  return RBCalibration::from_bytestream(tp.payload, pos);
}

// this can be possible, we can cast to py::object
// with py::cast. However, that might require 
// allocating on the heap. Let's be careful first
//py::object unpack(const TofPacket& tp) {
//  if (tp.packet_type == PacketType::RBCalibration) {
//    u64 pos = 0; 
//    return RBCalibration::from_bytestream(tp.payload, pos);
//  }
//  return py::int_(42);
//}

