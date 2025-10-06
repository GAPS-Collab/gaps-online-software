#include <nanobind/nanobind.h>
#include <nanobind/stl/string.h>
#include <nanobind/stl/vector.h>

#include "sd_legacy.hpp" 
#include "io.hpp" 
#include "caraspace.hpp"

int add(int a, int b) { return a + b; }

namespace nb = nanobind;
namespace g  = gondola;

NB_MODULE(gondola_cxx, m) {

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
    .def_prop_ro("time_a"  , &TofHit::get_time_a)
    .def_prop_ro("time_b"  , &TofHit::get_time_b)
    .def_prop_ro("charge_a", &TofHit::get_charge_a)
    .def_prop_ro("charge_b", &TofHit::get_charge_b)
    .def_prop_ro("peak_a"  , &TofHit::get_peak_a)
    .def_prop_ro("peak_b"  , &TofHit::get_peak_b)
    .def_prop_ro("edep"    , &TofHit::get_edep)
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
    .def_prop_ro("hits"    , &TofEvent::get_hits)
    .def_ro("header"       , &TofEvent::header)
    .def_prop_ro("rb_ids"  , &TofEvent::get_rbids)
    .def("to_string"       , &TofEvent::to_string);
}


