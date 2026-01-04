#include <nanobind/nanobind.h>
#include <nanobind/stl/string.h>
#include <nanobind/stl/vector.h>

#include "version.h"
#include "sd_legacy.hpp" 
#include "io.hpp" 
#include "caraspace.hpp"
#include "telemetry_dataclasses.hpp"
#include "io/telemetry_reader.hpp"
#include "calibration.h"

namespace nb  = nanobind;
namespace g   = gondola;
namespace gtl = Gaps::Telemetry;

NB_MODULE(gondola_cxx, m) {

  nb::enum_<Gaps::ProtocolVersion>(m, "ProtocolVersion") 
    .value("Unknown"           , Gaps::ProtocolVersion::Unknown)
    .value("V1"                , Gaps::ProtocolVersion::V1)
    .value("V2"                , Gaps::ProtocolVersion::V2)
    .value("V3"                , Gaps::ProtocolVersion::V3); 

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

  nb::class_<TofPacket>(m, "TofPacket")
    .def(nb::init<>())
    .def_static("from_bytestream", &TofPacket::from_bytestream)
    .def_prop_ro("packet_type", [](const TofPacket &p) {
       return p.packet_type;
    });

  // io 
  m.def("list_path_contents_sorted", &gondola::list_path_contents_sorted);
  
  nb::class_<Gaps::TofPacketReader>(m, "TofPacketReader")
    .def(nb::init<std::string>())
    //.def("rewind", &Gaps::TofPacketReader::rewind)
    .def("get_next_packet", [](Gaps::TofPacketReader &r) {
      return r.get_next_packet().unwrap();
    })
    .def_prop_ro("filename", &Gaps::TofPacketReader::get_filename);

  #ifdef BUILD_WITH_ROOT
  m.def("read_sd_legacy_example",&gondola::read_sd_legacy_example); 
  #endif 
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
  nb::class_<TofHit>(m, "TofHit")
    .def(nb::init<>())
    .def_ro("version"      , &TofHit::version)
    .def_prop_ro("time_a"  , &TofHit::get_time_a)
    .def_prop_ro("time_b"  , &TofHit::get_time_b)
    .def_prop_ro("charge_a", &TofHit::get_charge_a)
    .def_prop_ro("charge_b", &TofHit::get_charge_b)
    .def_prop_ro("peak_a"  , &TofHit::get_peak_a)
    .def_prop_ro("peak_b"  , &TofHit::get_peak_b)
    .def_prop_ro("edep"    , &TofHit::get_edep)
    .def_ro("event_t0"     , &TofHit::event_t0)
    .def_ro("paddle_id"    , &TofHit::paddle_id)
    .def("to_string", &TofHit::to_string)
    .def("__repr__", [](TofHit &h) {
      return "<NBWrapper" + h.to_string() + ">";
    }); 
  
  nb::class_<TofEventHeader>(m, "TofEventHeader")
    .def(nb::init<>())
    .def_ro("event_id"     , &TofEventHeader::event_id)
    .def("to_string"       , &TofEventHeader::to_string);

  nb::class_<TofEvent>(m, "TofEvent")
    .def(nb::init<>())
    .def_static("from_tofpacket",&TofEvent::from_tofpacket) 
    .def("normalize_hit_times", &TofEvent::normalize_hit_times)
    .def_prop_ro("hits"    , &TofEvent::get_hits)
    .def_ro("header"       , &TofEvent::header)
    .def_prop_ro("rb_ids"  , &TofEvent::get_rbids)
    .def("to_string"       , &TofEvent::to_string);

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
    .def_ro("header"       , &g::TelemetryPacket::header)
    .def_ro("payload"      , &g::TelemetryPacket::payload)
    .def("from_bytestream" , &g::TelemetryPacket::from_bytestream)
    .def("to_string"       , &g::TelemetryPacket::to_string)
    .def("__str__", [](const g::TelemetryPacket& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const g::TelemetryPacket& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<gtl::TrkHeader>(m, "TrkHeader")
    .def(nb::init<>())
    .def_static("from_bytestream", &gtl::TrkHeader::from_bytestream)
    .def("__str__", [](const gtl::TrkHeader& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const gtl::TrkHeader& self) {
        return nb::str("{}").format(self.to_string());
    });
  
    nb::class_<gtl::TrkHit>(m, "TrkHit")
    .def(nb::init<>())
    .def("__str__", [](const gtl::TrkHit& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const gtl::TrkHit& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<gtl::TrkEvent>(m, "TrkEvent")
    .def(nb::init<>())
    .def_ro("hits"      , &gtl::TrkEvent::hits)
    .def("__str__", [](const gtl::TrkEvent& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const gtl::TrkEvent& self) {
        return nb::str("{}").format(self.to_string());
    });

  nb::class_<gtl::TrkEventPacket>(m, "TrkEventPacket")
    .def(nb::init<>())
    .def_ro("events"      , &gtl::TrkEventPacket::events)
    .def_static("from_bytestream", [](Vec<u8> stream, usize pos) {
        return gtl::TrkEventPacket::from_bytestream(stream, pos).unwrap();
    })    
    .def("__str__", [](const gtl::TrkEventPacket& self) {
        return nb::str("{}").format(self.to_string());
    })
    .def("__repr__", [](const gtl::TrkEventPacket& self) {
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
    .def("rewind"                    , &g::TelemetryPacketReader::rewind);
  
  nb::class_<Gaps::Telemetry::MergedEvent>(m, "TelemetryEvent")
    .def(nb::init<>());

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


