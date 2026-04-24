#include <nanobind/nanobind.h>
#include <nanobind/stl/string.h>
#include <nanobind/stl/vector.h>
#include <nanobind/stl/shared_ptr.h>

#include "version.h"
#include "sd_legacy.hpp" 
#include "io.hpp" 
#include "caraspace.hpp"
#include "telemetry_dataclasses.hpp"
#include "io/telemetry_reader.hpp"
#include "calibration.h"

namespace nb  = nanobind;
namespace g   = gondola;

NB_MODULE(gondola_cxx, m) {

  nb::enum_<g::ProtocolVersion>(m, "ProtocolVersion") 
    .value("Unknown"           , g::ProtocolVersion::Unknown)
    .value("V1"                , g::ProtocolVersion::V1)
    .value("V2"                , g::ProtocolVersion::V2)
    .value("V3"                , g::ProtocolVersion::V3); 

  // packets 
  nb::enum_<PacketType>(m, "TofPacketType")
    .value("Unknown"           , PacketType::Unknown            )
    .value("Command"           , PacketType::Command            )
    .value("RBEvent"           , PacketType::RBEvent            )
    .value("TofEvent"          , PacketType::TofEvent           )
    .value("RBWaveform"        , PacketType::RBWaveform         )
    .value("TofEventSummary"   , PacketType::TofEventSummary    )
    .value("HeartBeat"         , PacketType::HeartBeat          )
    .value("Scalar"            , PacketType::Scalar             )
    .value("MasterTrigger"     , PacketType::MasterTrigger      )
    .value("RBHeader"          , PacketType::RBHeader           )
    .value("CPUMoniData"       , PacketType::CPUMoniData        )
    .value("MTBMoni"           , PacketType::MTBMoni            )
    .value("RBMoni"            , PacketType::RBMoni             )
    .value("PBMoniData"        , PacketType::PBMoniData         ) 
    .value("LTBMoniData"       , PacketType::LTBMoniData        )
    .value("PAMoniData"        , PacketType::PAMoniData         ) 
    .value("RBEventPayload"    , PacketType::RBEventPayload     )
    .value("RBEventMemoryView" , PacketType::RBEventMemoryView  )
    .value("RBCalibration"     , PacketType::RBCalibration      );
  
  nb::enum_<g::LTBThreshold>(m, "LTBThreshold")
    .value("NoHit"   , g::LTBThreshold::NoHit)
    .value("Hit"     , g::LTBThreshold::Hit)
    .value("Beta"    , g::LTBThreshold::Beta)
    .value("Veto"    , g::LTBThreshold::Veto) 
    .value("Unknown" , g::LTBThreshold::Unknown);

  nb::class_<TofPacket>(m, "TofPacket")
    .def(nb::init<>())
    .def_static("from_bytestream", &TofPacket::from_bytestream)
    .def_prop_ro("packet_type", [](const TofPacket &p) {
       return p.packet_type;
    });

  // io 
  m.def("list_path_contents_sorted", &gondola::list_path_contents_sorted);
  
  nb::class_<g::TofPacketReader>(m, "TofPacketReader")
    .def(nb::init<std::string>())
    //.def("rewind", &Gaps::TofPacketReader::rewind)
    .def("get_next_packet", [](g::TofPacketReader &r) {
      return r.get_next_packet().unwrap();
    })
    .def_prop_ro("filename", &g::TofPacketReader::get_filename);

  //#ifdef BUILD_CXX_WITH_ROOT
  m.def("read_sd_legacy_example",&g::read_sd_legacy_example); 
  nb::class_<g::SDRootReader>(m, "SDRootReader")
    .def(nb::init<std::string>())
    //.def("get_next_event", [](g::TofPacketReader &r) {
    //  return r.get_next_packet().unwrap();
    //})
    .def_ro("filename", &g::SDRootReader::filename)
    .def("get_event", &g::SDRootReader::get_event)
    .def_ro("nevents_total", &g::SDRootReader::nevents_total)
    .def("get_event_tof_energies", &g::SDRootReader::get_event_tof_energies)
    .def("get_event_trk_energies", &g::SDRootReader::get_event_trk_energies);

  nb::class_<g::SDRootWriter>(m, "SDRootWriter")
    .def(nb::init<std::string>())
    //.def("get_next_event", [](g::TofPacketReader &r) {
    //  return r.get_next_packet().unwrap();
    //})
    .def_ro("filename",      &g::SDRootWriter::filename)
    .def("add_event",        &g::SDRootWriter::add_event)
    .def("write_sdpar",      &g::SDRootWriter::write_sdpar)
    .def_ro("nevents_total", &g::SDRootWriter::nevents_total);
  //#endif 
  
  // caraspace
  nb::enum_<g::CRFrameObjectType>(m, "CRFrameObjectType")
     .value("Unknown",         g::CRFrameObjectType::Unknown)
     .value("TofPacket",       g::CRFrameObjectType::TofPacket)
     .value("TelemetryPacket", g::CRFrameObjectType::TelemetryPacket); 

  nb::class_<g::CRFrameObject>(m, "CRFrameObject")
     .def("to_bytestream", &g::CRFrameObject::to_bytestream); 

  nb::class_<g::CRReader>(m, "CRReader")
    .def(nb::init<std::string>())
    .def("get_next_frame", &g::CRReader::get_next_frame) 
    .def_prop_ro("filenames", &g::CRReader::get_filenames);
  
  nb::class_<g::CRFrame>(m, "CRFrame") 
    .def(nb::init<>())  
    .def("get_tofpacket", [](g::CRFrame &f, const std::string &name) {
      return f.get_tofpacket(name).unwrap();
    })
    .def("to_string", &g::CRFrame::to_string);

  // events  
  nb::class_<g::TofHit>(m, "TofHit")
    .def(nb::init<>())
    .def_ro("version"        , &g::TofHit::version)
    //.def_prop_ro("time_a"    , &g::TofHit::get_time_a)
    //.def_prop_ro("time_b"    , &g::TofHit::get_time_b)
    //.def_prop_ro("charge_a"  , &g::TofHit::get_charge_a)
    //.def_prop_ro("charge_b"  , &g::TofHit::get_charge_b)
    //.def_prop_ro("peak_a"    , &g::TofHit::get_peak_a)
    //.def_prop_ro("peak_b"    , &g::TofHit::get_peak_b)
    .def_rw("time_a"         , &g::TofHit::time_a_f32)
    .def_rw("time_b"         , &g::TofHit::time_b_f32)
    .def_rw("charge_a"       , &g::TofHit::charge_a_f32)
    .def_rw("charge_b"       , &g::TofHit::charge_b_f32)
    .def_rw("peak_a"         , &g::TofHit::peak_a_f32)
    .def_rw("peak_b"         , &g::TofHit::peak_b_f32)
    .def_prop_ro("edep"      , &g::TofHit::get_edep)
    .def_prop_ro("x0"        , &g::TofHit::get_x_pos)
    .def_prop_ro("t0_uncorr" , &g::TofHit::get_t0_relative)
    .def_prop_ro("obeys_causality", &g::TofHit::obeys_causality)
    .def_prop_ro("tot_low_a" , &g::TofHit::get_tot_low_a)
    .def_prop_ro("tot_low_b" , &g::TofHit::get_tot_low_b)
    .def_prop_ro("tot_high_a", &g::TofHit::get_tot_high_a)
    .def_prop_ro("tot_high_b", &g::TofHit::get_tot_high_b)
    .def_prop_ro("slp_low_a" , &g::TofHit::get_tot_slp_low_a)
    .def_prop_ro("slp_low_b" , &g::TofHit::get_tot_slp_low_b)
    .def_prop_ro("slp_high_a", &g::TofHit::get_tot_slp_high_a)
    .def_prop_ro("slp_high_b", &g::TofHit::get_tot_slp_high_b)
    .def_rw("paddle_len"     , &g::TofHit::paddle_len)
    .def_rw("event_t0"       , &g::TofHit::event_t0)
    .def_rw("paddle_id"      , &g::TofHit::paddle_id)
    .def("to_string"         , &g::TofHit::to_string)
    .def("__repr__", [](g::TofHit &h) {
      return "<NBWrapper" + h.to_string() + ">";
    }); 
  
  nb::class_<g::TofEvent>(m, "TofEvent")
    .def(nb::init<>())
    .def_static("from_tofpacket", &g::TofEvent::from_tofpacket) 
    .def_ro("dsi_j_mask"        , &g::TofEvent::dsi_j_mask)
    .def("normalize_hit_times"  , &g::TofEvent::normalize_hit_times)
    .def_prop_ro("hits"         , &g::TofEvent::get_hits)
    .def_prop_ro("rb_ids"       , &g::TofEvent::get_rbids)
    .def_prop_ro("timestamp48"  , &g::TofEvent::get_timestamp48)
    .def_prop_ro("rb_link_ids"  , &g::TofEvent::get_rb_link_ids) 
    .def_prop_ro("trigger_hits" , &g::TofEvent::get_trigger_hits) 
    .def_prop_ro("trigger_sources", &g::TofEvent::get_trigger_sources)
    .def("to_string"            , &g::TofEvent::to_string);
  
  nb::class_<g::TofEventSummary>(m, "TofEventSummary")
    .def(nb::init<>())
    .def_static("from_tofpacket"   , &g::TofEventSummary::from_tofpacket) 
    .def_rw("event_id"             , &g::TofEventSummary::event_id)
    .def_rw("run_id"               , &g::TofEventSummary::run_id)
    .def_rw("dsi_j_mask"           , &g::TofEventSummary::dsi_j_mask)
    .def_rw("channel_masks"        , &g::TofEventSummary::channel_mask)
    //.def("normalize_hit_times"  , &g::TofEvent::normalize_hit_times)
    //.def_prop_ro("hits"         , &g::TofEvent::get_hits)
    .def_rw("hits"                , &g::TofEventSummary::hits)
    .def_prop_ro("timestamp48"     , &g::TofEventSummary::get_timestamp48)
    .def_prop_ro("rb_link_ids"     , &g::TofEventSummary::get_rb_link_ids) 
    //.def_prop_ro("trigger_hits"   , &g::TofEventSummary::get_trigger_hits) 
    .def_prop_ro("trigger_pids"   , &g::TofEventSummary::get_trigger_pids)
    .def_prop_ro("trigger_hits"   , [](g::TofEventSummary &self) {
      auto thits        = self.get_trigger_hits();
      Vec<Vec<int>> thits_py = {};
      for (auto const &h : thits) {
        Vec<int> h_py    = {};
        h_py.push_back((int)std::get<0>(h));
        h_py.push_back((int)std::get<1>(h));
        h_py.push_back((int)std::get<2>(h));
        h_py.push_back((int)std::get<3>(h));
        thits_py.push_back(h_py);
      } 
      return thits_py;
    })
    .def_prop_ro("trigger_sources", &g::TofEventSummary::get_trigger_sources)
    .def_rw("trigger_sources_bytes", &g::TofEventSummary::trigger_sources)
    .def("to_string"              , &g::TofEventSummary::to_string);

  //---------------------------------------------------------
  // Telemetry packets & reader
  nb::enum_<g::TelemetryPacketType>(m, "TelemetryPacketType")
    .value("Unknown"           , g::TelemetryPacketType::Unknown            )
    .value("CardHKP"           , g::TelemetryPacketType::CardHKP            )
    .value("CoolingHK"         , g::TelemetryPacketType::CoolingHK          )
    .value("PDUHK"             , g::TelemetryPacketType::PDUHK              )
    .value("Tracker"           , g::TelemetryPacketType::Tracker            )
    .value("TrackerDAQCntr"    , g::TelemetryPacketType::TrackerDAQCntr     )
    .value("GPS"               , g::TelemetryPacketType::GPS                )
    .value("TrkTempLeak"       , g::TelemetryPacketType::TrkTempLeak        )
    .value("BoringEvent"       , g::TelemetryPacketType::BoringEvent        )
    .value("RBWaveform"        , g::TelemetryPacketType::RBWaveform         )
    .value("AnyTofHK"          , g::TelemetryPacketType::AnyTofHK           )
    .value("GcuEvtBldSettings" , g::TelemetryPacketType::GcuEvtBldSettings  )
    .value("LabJackHK"         , g::TelemetryPacketType::LabJackHK          )
    .value("MagHK"             , g::TelemetryPacketType::MagHK              )
    .value("GcuMon"            , g::TelemetryPacketType::GcuMon             )
    .value("InterestingEvent"  , g::TelemetryPacketType::InterestingEvent   )
    .value("NoGapsTriggerEvent", g::TelemetryPacketType::NoGapsTriggerEvent )
    .value("NoTofDataEvent"    , g::TelemetryPacketType::NoTofDataEvent     )
    .value("Ack"               , g::TelemetryPacketType::Ack                )     
    .value("TmP33"             , g::TelemetryPacketType::TmP33              )
    .value("TmP34"             , g::TelemetryPacketType::TmP34              )
    .value("TmP37"             , g::TelemetryPacketType::TmP37              )
    .value("TmP38"             , g::TelemetryPacketType::TmP38              )
    .value("TmP55"             , g::TelemetryPacketType::TmP55              )
    .value("TmP64"             , g::TelemetryPacketType::TmP64              )
    .value("TmP96"             , g::TelemetryPacketType::TmP96              )
    .value("TmP214"            , g::TelemetryPacketType::TmP214             )
    .value("HeatHVLVSettings"  , g::TelemetryPacketType::HeatHVLVSettings   )
    .value("LabJackSettings"   , g::TelemetryPacketType::LabjackSettings    )
    .value("SurvivalPacket"    , g::TelemetryPacketType::SurvivalPacket     )
    .value("GcuMonHKAddendum"  , g::TelemetryPacketType::GcuMonHKAddendum   )
    .value("TeleMainSettings"  , g::TelemetryPacketType::TeleMainSettings   )
    .value("PacketStats"       , g::TelemetryPacketType::PacketStats        )
    .value("DecimationSettings", g::TelemetryPacketType::DecimationSettings )
    .value("RPiHKP"            , g::TelemetryPacketType::RPiHKP             )
    .value("GcuEvtBuilderStats", g::TelemetryPacketType::GcuEvtBuilderStats )
    .value("SipGpsPosition"    , g::TelemetryPacketType::SipGpsPosition     )
    .value("SipGpsTime"        , g::TelemetryPacketType::SipGpsTime         )
    .value("SipPressure"       , g::TelemetryPacketType::SipPressure        )
    .value("RatePacket"        , g::TelemetryPacketType::RatePacket         )
    .value("AnyTrackerHK"      , g::TelemetryPacketType::AnyTrackerHK       );
  
  nb::class_<g::TelemetryPacketHeader>(m, "TelemetryPacketHeader")
    .def(nb::init<>())
    .def_prop_ro("gcutime" , &g::TelemetryPacketHeader::get_gcutime)
    .def_ro("packet_type"  , &g::TelemetryPacketHeader::ptype)
    .def_ro("counter"      , &g::TelemetryPacketHeader::counter)
    .def_ro("length"       , &g::TelemetryPacketHeader::length)
    .def_ro("checksum"     , &g::TelemetryPacketHeader::checksum)
    .def("to_string"       , &g::TelemetryPacketHeader::to_string)
    .def("__str__", [](const g::TelemetryPacketHeader& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TelemetryPacketHeader& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<g::TelemetryPacket>(m, "TelemetryPacket")
    .def(nb::init<>())
    .def_ro("header"          , &g::TelemetryPacket::header)
    .def_ro("payload"         , &g::TelemetryPacket::payload)
    .def_prop_ro("is_event_packet" , &g::TelemetryPacket::is_event_packet)
    .def("get_paddle_len"     , [](const g::TelemetryPacket& self, u8 pid) {
      auto paddle = self.paddles->at(pid);
      return paddle.length;
    })
    .def("from_bytestream"    , &g::TelemetryPacket::from_bytestream)
    .def("to_string"          , &g::TelemetryPacket::to_string)
    .def("__str__", [](const g::TelemetryPacket& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TelemetryPacket& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<g::TrkHeader>(m, "TrkHeader")
    .def(nb::init<>())
    .def_static("from_bytestream", &g::TrkHeader::from_bytestream)
    .def("__str__", [](const g::TrkHeader& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TrkHeader& self) {
        return nb::str("{}").format(self.to_string());
    });
  
  nb::class_<g::TrkHit>(m, "TrkHit")
    .def(nb::init<>())
    .def_rw("layer"           , &g::TrkHit::layer)
    .def_rw("row"             , &g::TrkHit::row)
    .def_rw("module"          , &g::TrkHit::module)
    .def_rw("channel"         , &g::TrkHit::channel)
    .def_rw("adc"             , &g::TrkHit::adc)
    .def_rw("oscillator"      , &g::TrkHit::oscillator)
    .def_rw("energy"          , &g::TrkHit::energy)
    .def_rw("asic_event_code" , &g::TrkHit::asic_event_code)
    
    .def("__str__", [](const g::TrkHit& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TrkHit& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<g::TrkEvent>(m, "TrkEvent")
    .def(nb::init<>())
    .def_ro("hits"      , &g::TrkEvent::hits)
    .def("__str__", [](const g::TrkEvent& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TrkEvent& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<g::TrkEventPacket>(m, "TrkEventPacket")
    .def(nb::init<>())
    .def_ro("events"      , &g::TrkEventPacket::events)
    .def_static("from_bytestream", [](Vec<u8> stream, usize pos) {
        return g::TrkEventPacket::from_bytestream(stream, pos).unwrap();
    })    
    .def("__str__", [](const g::TrkEventPacket& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TrkEventPacket& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<g::TelemetryPacketReader>(m, "TelemetryPacketReader")
    .def(nb::init<std::string>())
    .def_prop_ro("filenames"         , &g::TelemetryPacketReader::get_filenames)
    .def("get_next_packet"           , &g::TelemetryPacketReader::get_next_packet)
    .def_prop_ro("exhausted"         , &g::TelemetryPacketReader::is_exhausted)
    .def_prop_ro("get_packet_index"  , &g::TelemetryPacketReader::get_packet_index)
    .def("print_packet_index"        , &g::TelemetryPacketReader::print_packet_index)
    .def("cache_all_packets"         , &g::TelemetryPacketReader::cache_all_packets)
    .def("count_packets"             , &g::TelemetryPacketReader::count_packets)
    .def("get_paddle_len"     , [](const g::TelemetryPacketReader& self, u8 pid) {
      auto paddle = self.paddles->at(pid);
      return paddle.length;
    })
    .def("rewind"                    , &g::TelemetryPacketReader::rewind);
  
  nb::class_<g::TelemetryEvent>(m, "TelemetryEvent")
    .def(nb::init<>())
    .def_static("from_telemetrypacket", [](g::TelemetryPacket &pack) {
      auto data = g::TelemetryEvent::from_telemetrypacket(pack);
      if (data.is_ok()) {
        return data.unwrap();
      }  
      throw nb::value_error("Error when unpacking TelemetryEvent!");
    })
    .def_static("from_bytestream", [](Vec<u8> stream, usize pos) {
      auto data = g::TelemetryEvent::from_bytestream(stream, pos);
      if (data.is_ok()) {
        return data.unwrap();
      }  
      throw nb::value_error("Error when unpacking TelemetryEvent!");
      //return g::TelemetryEvent::from_bytestream(stream, pos).unwrap();
    })
    .def_rw("event_id"           , &g::TelemetryEvent::event_id)
    .def_rw("tof"                , &g::TelemetryEvent::tof_event)
    .def_rw("tracker"            , &g::TelemetryEvent::trk_hits)
    .def("__str__", [](const g::TelemetryEvent& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TelemetryEvent& self) {
        return nb::str("{}").format(self.to_string());
    });

  // Spike cleaning functions
  m.def("spike_cleaning_drs4", &g::spike_cleaning_drs4, 
        nb::arg("wf"), nb::arg("tCell"), nb::arg("spikes"),
        "Original DRS4 spike cleaning from the DRS4 manual");
  
  m.def("spike_cleaning_simple", &g::spike_cleaning_simple, 
        nb::arg("voltages"), nb::arg("calibrated") = true,
        "Simpler spike cleaning version (by Jamie)");
  
  m.def("spike_cleaning_all", &g::spike_cleaning_all, 
        nb::arg("voltages"), nb::arg("calibrated") = true,
        "Jamie's simpler version with single-width spike correction");
}


