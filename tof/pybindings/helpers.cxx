#include "tof_typedefs.h"
#include "serialization.h"
#include "io.hpp"
#include "helpers.hpp"

#include "spdlog/spdlog.h"
#include <pybind11/pybind11.h>

namespace py = pybind11;

bytestream wrap_encode_ushort(u16 value, u32 start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort_rev(value, stream, start_pos);
  return stream;
}


/***********************************************/

bytestream wrap_u32_to_le_bytes(u32 value) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  u32_to_le_bytes(value, stream, 0);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64(value, stream, start_pos);
  return stream;
}

/***********************************************/

Vec<TofPacket> wrap_get_tofpackets_from_file(const String filename) {
  return get_tofpackets(filename);
}

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

String tofevent_to_string(const TofEvent &event) {
  String repr = "<TofEvent\n";
  //repr += "\thead "      + std::to_string(event.head )      + "\n" ;
  repr += "\tn missing hits: "    + std::to_string(event.missing_hits.size() )    + "\n" ;
  repr += "\tn RB Events: "       + std::to_string(event.rb_events.size() )       + "\n" ;
  repr += "\tn RB Monis "         + std::to_string(event.rb_moni_data.size() )       + "\n" ;
  //repr += "\tn PaddlePackets "    + std::to_string(event.dna )       + "\n" ;
  //repr += "\ttail "      + std::to_string(event.tail)       ;
  repr += ">";
  return repr;
}

/***********************************************/

String rbevent_to_string(const RBEvent &event) {
  String repr = "<RBEvent\n";
  //repr += "\thead "      + std::to_string(event.head )      + "\n" ;
  //repr += "\tn missing hits: "    + std::to_string(event.missing_hits.size() )    + "\n" ;
  //repr += "\tn RB Events: "       + std::to_string(event.rb_events.size() )       + "\n" ;
  //repr += "\tn RB Monis "         + std::to_string(event.rb_moni_data.size() )       + "\n" ;
  //repr += "\tn PaddlePackets "    + std::to_string(event.dna )       + "\n" ;
  //repr += "\ttail "      + std::to_string(event.tail)       ;
  repr += ">";
  return repr;
}

/***********************************************/

String mastertriggerevent_to_string(const MasterTriggerEvent &event) {
  String repr = "<MasterTriggerEvent\n";
  repr += "\t event_id      :" + std::to_string(event.event_id     ) + "\n" ; 
  repr += "\t timestamp     :" + std::to_string(event.timestamp    ) + "\n" ; 
  repr += "\t tiu_timestamp :" + std::to_string(event.tiu_timestamp) + "\n" ; 
  repr += "\t tiu_gps_32    :" + std::to_string(event.tiu_gps_32   ) + "\n" ; 
  repr += "\t tiu_gps_16    :" + std::to_string(event.tiu_gps_16   ) + "\n" ; 
  repr += "\t n_paddles     :" + std::to_string(event.n_paddles    ) + "\n" ; 
  repr += "\t [DSI/J] 1/1 - 1/2 - 1/3 - 1/4 - 1/5 - 2/1 - 2/2 - 2/3 - 2/4 - 2/5 - 3/1 - 3/2 - 3/3 - 3/4 - 3/5 - 4/1 - 4/2 - 4/3 - 4/4 - 4/5 \n";
  Vec<u8> hit_boards = Vec<u8>();
  HashMap<u8, String> dsi_j = HashMap<u8, String>();
  dsi_j[0] = "1/1";
  dsi_j[1] = "1/2";
  dsi_j[2] = "1/3";
  dsi_j[3] = "1/4";
  dsi_j[4] = "1/5";
  dsi_j[5] = "2/1";
  dsi_j[6] = "2/2";
  dsi_j[7] = "2/3";
  dsi_j[8] = "2/4";
  dsi_j[9] = "2/5";
  dsi_j[10] = "3/1";
  dsi_j[11] = "3/2";
  dsi_j[12] = "3/3";
  dsi_j[13] = "3/4";
  dsi_j[14] = "3/5";
  dsi_j[15] = "4/1";
  dsi_j[16] = "4/2";
  dsi_j[16] = "4/3";
  dsi_j[17] = "4/4";
  dsi_j[19] = "4/5";
  repr += "\t         ";
  for (usize k=0;k<N_LTBS;k++) {
    if (event.board_mask[k]) {
      repr += "-X-   ";
      hit_boards.push_back(k);
    } else {
      repr += "-0-   ";
    }
  }
  repr += "\n\t == == HITS [CH] == ==\n";
  for (auto k : hit_boards) {
    repr += "\t DSI/J " + dsi_j[k] + "\t=> ";
    for (usize j=0;j<N_CHN_PER_LTB;j++) {
      if (event.hits[k][j]) {
        repr += " " + std::to_string(j + 1) + " ";
      } else {
        continue;
        //repr += " N.A. ";
      } 
    }
    repr += "\n";
  }  
  repr += ">";
  return repr;
}

Vec<f32> wrap_rbcalibration_voltages_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }

  return calib.voltages(event, channel);
}

Vec<f32> wrap_rbcalibration_voltages_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.voltages(event, channel);
}

Vec<f32> wrap_rbcalibration_nanoseconds_rbevent(const RBCalibration& calib, const RBEvent& event, const u8 channel) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.nanoseconds(event, channel);
}

Vec<f32> wrap_rbcalibration_nanoseconds_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, const u8 channel) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.nanoseconds(event, channel);
}

Vec<Vec<f32>> wrap_rbcalibration_voltages_allchan_rbevent(const RBCalibration& calib, const RBEvent& event, bool spike_cleaning) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.voltages(event, spike_cleaning);
}

Vec<Vec<f32>> wrap_rbcalibration_voltages_allchan_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event, bool spike_cleaning) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.voltages(event, spike_cleaning);
}

Vec<Vec<f32>> wrap_rbcalibration_nanoseconds_allchan_rbevent(const RBCalibration& calib, const RBEvent& event) {
  if (event.header.rb_id != calib.rb_id) {
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.header.rb_id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.nanoseconds(event);
}

Vec<Vec<f32>> wrap_rbcalibration_nanoseconds_allchan_rbeventmemoryview(const RBCalibration& calib, const RBEventMemoryView& event) {
  if (event.id != calib.rb_id) {
    spdlog::error("This is the wrong calibration!");
    String message = "This is calibration for board " + std::to_string(calib.rb_id) + " but the event is from board " + std::to_string(event.id);
    PyErr_SetString(PyExc_ValueError, message.c_str());
    throw py::error_already_set();
  }
  return calib.nanoseconds(event);
}


