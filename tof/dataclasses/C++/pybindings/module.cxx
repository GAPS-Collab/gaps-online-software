#include <fstream>

#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/complex.h>
#include <pybind11/functional.h>
#include <pybind11/chrono.h>
#include <pybind11/numpy.h>

#include "packets/tof_packet.h"
#include "packets/CommandPacket.h"
#include "packets/monitoring.h"
#include "events/tof_event_header.hpp"

#include "legacy.h"
#include "io.hpp"
#include "serialization.h"
#include "calibration.h"
#include "events.h"

#include "tof_typedefs.h"

#include "helpers.hpp"

using namespace GAPS;
using namespace pybind11::literals;
namespace py = pybind11;

/********************/
// helpers


std::string tof_command_to_str(const TofCommand &cmd) {
 switch (cmd) {
   case TofCommand::PowerOn               : {return "PowerOn"              ;}
   case TofCommand::PowerOff              : {return "PowerOff"             ;}
   case TofCommand::PowerCycle            : {return "PowerCycle"           ;} 
   case TofCommand::RBSetup               : {return "RBSetup"              ;}
   case TofCommand::SetThresholds         : {return "SetThresholds"        ;} 
   case TofCommand::SetMtConfig           : {return "SetMtConfig"          ;}
   case TofCommand::StartValidationRun    : {return "StartValidationRun"   ;}
   case TofCommand::RequestWaveforms      : {return "RequestWaveforms"     ;}
   case TofCommand::DataRunStart          : {return "DataRunStart"         ;}
   case TofCommand::DataRunEnd            : {return "DataRunEnd"           ;}
   case TofCommand::VoltageCalibration    : {return "VoltageCalibration"   ;}
   case TofCommand::TimingCalibration     : {return "TimingCalibration"    ;}
   case TofCommand::CreateCalibrationFile : {return "CreateCalibrationFile";}
   case TofCommand::RequestEvent          : {return "RequestEvent"         ;}
   case TofCommand::RequestMoni           : {return "RequestMoni"          ;}
   case TofCommand::UnspoolEventCache     : {return "UnspoolEventCahce"    ;}
   case TofCommand::StreamAnyEvent        : {return "StreamAnyEvent"       ;} 
   case TofCommand::Unknown               : {return "Unknown"              ;}
 } // end case   
 return "Unknown";
}

std::string tof_response_to_str(const TofResponse &cmd) {
 switch (cmd) {
   case TofResponse::Success            : {return "Success"     ;}
   case TofResponse::GeneralFailure     : {return "GeneralFailure" ;}
   case TofResponse::EventNotReady      : {return "EventNotReady" ;}
   case TofResponse::SerializationIssue : {return "SerializationIssue" ;}
 } // end case   
 return "Unknown";
}

Vec<Vec<f64>> remove_spikes_helper(u16 stop_cell,
                                 Vec<Vec<f64>> waveforms) {
 f64 wf [NCHN][NWORDS];
 i32 spikes[NWORDS];
 Vec<Vec<f64>> unspiked;
 for (size_t ch=0; ch<NCHN; ch++) {
   unspiked.push_back({});
   for (size_t n=0; n<NWORDS; n++) {
     wf[ch][n] = waveforms[ch][n];
   }

 }
 RemoveSpikes(wf, stop_cell, spikes);
 for (size_t ch=0; ch<NCHN; ch++) {
   for (size_t n=0; n<NWORDS; n++) {
     unspiked[ch].push_back(wf[ch][n]);
   } 
 }
 return unspiked;
}

/********************/

double calculate_pedestal_helper(Vec<f64> wave,
                                 Vec<f64> time,
                                 size_t ch)
{
  double* wave_arr = wave.data();
  double* time_arr = time.data();
  Waveform waveform = Waveform(wave_arr, time_arr, ch, 0);
  waveform.SetPedBegin(10); // 10-100                               
  waveform.SetPedRange(50);
  waveform.CalcPedestalRange();
  //waveform.SubtractPedestal();
  return waveform.GetPedestal();
}

/********************/


PYBIND11_MODULE(gaps_tof, m) {
    m.doc() = "Gaps-online-software Python wrapper for C++ API. gaps-online-software is a software suite designed to read out (mainly) online data from the TOF subsystem of the GAPS experiment. The code has several APIs, this code here wraps the C++ API. Please find the github repo at https://github.com/GAPS-Collab/gaps-online-software to report bugs/issues.";
    m.attr("__version__") = "0.10.0";

    py::class_<Gaps::TofPacketReader>(m, "TofPacketReader") 
      .def(py::init<String>())  
      .def("get_next_packet", &Gaps::TofPacketReader::get_next_packet,
                              "iterate over the packets and get the next packet from the file")
      .def("__repr__",        [](const Gaps::TofPacketReader &reader) {
                                  return "<TofPacketReader : "
                                  + reader.get_filename() + ">";
                                  }) 
    ;
  
    py::enum_<EventStatus>(m, "EventStatus") 
      .value("Unknown"           , EventStatus::Unknown)
      .value("Crc32Wrong"        , EventStatus::Crc32Wrong)
      .value("TailWrong"         , EventStatus::TailWrong)
      .value("IncompleteReadout" , EventStatus::IncompleteReadout)
      .value("Perfect"           , EventStatus::Perfect)

    ;
    py::enum_<TofCommand>(m, "TofCommand")
      .value("PowerOn"              ,TofCommand::PowerOn) 
      .value("PowerOff"             ,TofCommand::PowerOff) 
      .value("PowerCycle"           ,TofCommand::PowerCycle) 
      .value("RBSetup"              ,TofCommand::RBSetup) 
      .value("SetThresholds"        ,TofCommand::SetThresholds) 
      .value("SetMtConfig"          ,TofCommand::SetMtConfig) 
      .value("StartValidationRun"   ,TofCommand::StartValidationRun) 
      .value("RequestWaveforms"     ,TofCommand::RequestWaveforms) 
      .value("DataRunStart"         ,TofCommand::DataRunStart) 
      .value("DataRunEnd"           ,TofCommand::DataRunEnd)    
      .value("VoltageCalibration"   ,TofCommand::VoltageCalibration) 
      .value("TimingCalibration"    ,TofCommand::TimingCalibration)
      .value("CreateCalibrationFile",TofCommand::CreateCalibrationFile) 
      .value("RequestEvent"         ,TofCommand::RequestEvent) 
      .value("RequestMoni"          ,TofCommand::RequestMoni)
      .value("UnspoolEventCache"    ,TofCommand::UnspoolEventCache)
      .value("StreamAnyEvent"       ,TofCommand::StreamAnyEvent) 
      .value("Unknown"              ,TofCommand::Unknown) 
      //.export_values();
      ;
    
    py::enum_<TofResponse>(m, "TofResponse")
      .value("Success"                 ,TofResponse::Success) 
      .value("GeneralFailure"          ,TofResponse::GeneralFailure) 
      .value("EventNotReady"           ,TofResponse::EventNotReady) 
      .value("EventSerializationIssue" ,TofResponse::SerializationIssue) 
      .value("Unknown"                 ,TofResponse::Unknown) 
      //.export_values()
    ;

    py::class_<RBEventHeader>(m, "RBEventHeader", "The event header contains the event id, information about active channels, temperatures, trigger stop cell etc. Basically everythin except channel adc data.")
      .def(py::init())
      .def("from_bytestream"             , &RBEventHeader::from_bytestream, "Deserialize from a list of bytes")
      .def("get_active_data_channels"    , &RBEventHeader::get_active_data_channels, "Get a list of active channels, excluding ch9. Channel9 will (usually) always be on, as long as a single data channel is switched on as well.")
      .def("get_fpga_temp"               , &RBEventHeader::get_fpga_temp, "The FPGA temperature in C")
      .def_property_readonly("timestamp48"  , &RBEventHeader::get_timestamp48, "The complete 48bit timestamp, derived from the RB clock (usually 33MHz)")
      .def("get_n_datachan"              , &RBEventHeader::get_n_datachan)
      .def("get_nchan"                   , &RBEventHeader::get_nchan) 
      .def("get_channels"                , &RBEventHeader::get_channels) 
      .def_readonly("channel_mask"       , &RBEventHeader::channel_mask)   
      .def("has_ch9"                     , &RBEventHeader::has_ch9, "Ch9 is available"     )
      .def_readonly("stop_cell"          , &RBEventHeader::stop_cell   )   
      .def_property_readonly("is_locked"                   , &RBEventHeader::is_locked,
           "Is the RB loceked?"   )   
      .def_property_readonly("is_locked_last_sec"          , &RBEventHeader::is_locked_last_sec,
           "Has the RB been locked continuously throughout the last second?")   
      .def_property_readonly("lost_lock"                   , &RBEventHeader::lost_lock   )   
      .def_property_readonly("lost_lock_last_sec"          , &RBEventHeader::lost_lock_last_sec)   
      .def_property_readonly("lost_trigger"                , &RBEventHeader::drs_lost_trigger)   
      .def_property_readonly("event_fragment"              , &RBEventHeader::is_event_fragment) 
      .def("get_sine_fit"                , &RBEventHeader::get_sine_fit,
            "Get the result (amp,freq,phase) of an online sine fit to ch9")  
      .def_readonly("fpga_temp"          , &RBEventHeader::fpga_temp   )   
      .def_readonly("event_id"           , &RBEventHeader::event_id    )   
      .def_readonly("rb_id"              , &RBEventHeader::rb_id       )   
      .def_readonly("timestamp32"        , &RBEventHeader::timestamp32, 
              "LSB of the 48bit timestamp. Fast component")   
      .def_readonly("timestamp16"        , &RBEventHeader::timestamp16,
              "MSB of the 48bit timestamp. Slow component")   
      .def("get_timestamp48"             , &RBEventHeader::get_timestamp48,
              "Combined 48bit timestamp as generated by RB clock synth.")
      .def("__repr__",        [](const RBEventHeader &h) {
                                   return h.to_string();
                                 })
    ;

    py::class_<RBEvent>(m, "RBEvent", "RBEvent contains an event header for this specific board as well as a (flexible) number of adc channels")
        .def(py::init())
        .def_readonly("status"              ,&RBEvent::status,
                "Flag indicating if the event is ok or if there were issues, e.g. during readout")
        .def_readonly("header"              ,&RBEvent::header,
                "RBEventHeader stores all information which is NOT channel data")
        .def_readonly("hits"                ,&RBEvent::hits)
        .def_static("calc_baseline"         ,&RBEvent::calc_baseline,
                "Calculate baseline for a specific channel in min/max range")  
        .def("get_channel_adc"              ,&RBEvent::get_channel_adc,
                                             "Get the ADC values for a specific channel. Channel ids go from 1-9",
                                             py::arg("channel"),
                                             pybind11::return_value_policy::reference_internal)
        .def("from_bytestream"              ,&RBEvent::from_bytestream,
                                             "Decode the RBEvent from a list of bytes")
    .def("__repr__",          [](const RBEvent &ev) {
                                 return ev.to_string(); 
                              }) 

    ;

    py::class_<RBWaveform>(m, "RBWaveform", "RBWaveform is a single waveform for a single RB channel and needs to be assembled back to RBEvents/TofEvents")
      .def(py::init())
      .def_readonly("rb_id"              ,&RBWaveform::rb_id,
           "RB id (between 1-50). Internal TOF identifier for RB")
      .def_readonly("rb_channel"         ,&RBWaveform::rb_channel,
           "Channel id (between 0-9). Internal TOF identifier for RB channel. Each paddle has 2 channels")
      .def_readonly("event_id"           ,&RBWaveform::event_id,
           "Global event id")
      .def_readonly("stop_cell"          ,&RBWaveform::stop_cell,
           "Global event id")
      .def_readonly("adc"                ,&RBWaveform::adc,
           "Channel adc")
      .def("from_bytestream"            , &RBWaveform::from_bytestream,
           "deserialize RBWaveform from a list of bytes")
      .def("__repr__",        [](const RBWaveform &wf) {
                                 return wf.to_string(); 
                                 }) 
    ;

    py::class_<TofEventSummary>(m, "TofEventSummary", "The short version of a TofEvent. Only summary information + hits")
      .def(py::init())
      .def_readonly("status"            ,&TofEventSummary::status,
           "Event Status byte")
      .def_readonly("quality"           ,&TofEventSummary::quality,
           "Event Quality byte")
      .def_readonly("trigger_sources"   ,&TofEventSummary::trigger_sources,
           "Active Trigger Sources")
      .def_readonly("n_trigger_paddles" ,&TofEventSummary::n_trigger_paddles,
           "Number of triggered paddles (hits)")
      .def_readonly("event_id"          ,&TofEventSummary::event_id,
           "Master trigger event id")
      .def_readonly("timestamp16"       ,&TofEventSummary::timestamp16,
           "Timestamp 16bit (slow)")
      .def_readonly("timestamp32"       ,&TofEventSummary::timestamp32,
           "Timestamp 32bit (fast)")
      .def_readonly("primary_beta"      ,&TofEventSummary::primary_beta,
           "Beta from online track reconstruction. This might be a crude approximation")
      .def("get_trigger_hits"           , &TofEventSummary::get_trigger_hits, "Get the hits in dsi,j,channel, threshold format whcih formed the trigger")
      .def("get_rb_link_ids"            , &TofEventSummary::get_rb_link_ids, "Get the Link IDS of the RBs with expected hits within the trigger integration window") 
      .def("get_trigger_sources"        , &TofEventSummary::get_trigger_sources, "Return all active triggers for this event") 
      .def_property_readonly("timestamp48",   &TofEventSummary::get_timestamp48,
           "Complete timestamp (48 bits)")
      
      //.def_property_readonly("primary_beta",  &TofEventSummary::get_prim,
      //     "Complete timestamp (48 bits)")
      // primary charge and beta missing
      .def_readonly("hits"              ,&TofEventSummary::hits,
           "TofHits")
      .def("from_bytestream"            , &TofEventSummary::from_bytestream,
           "deserialize TofEventSummary from a list of bytes")
      .def("__repr__",        [](const TofEventSummary &tes) {
                                 return tes.to_string(); 
                                 }) 
    ;

    py::class_<MasterTriggerEvent>(m, "MasterTriggerEvent", "The MasterTriggerEvent contains the information from the MTB.")
      .def(py::init())
      .def("from_bytestream", &MasterTriggerEvent::from_bytestream, "Deserialize from a list of bytes")
      .def("get_trigger_hits"         , &MasterTriggerEvent::get_trigger_hits, "Get the hits in dsi,j,channel, threshold format whcih formed the trigger")
      .def("get_rb_link_ids"          , &MasterTriggerEvent::get_rb_link_ids, "Get the Link IDS of the RBs with expected hits within the trigger integration window") 
      .def("get_trigger_sources"      , &MasterTriggerEvent::get_trigger_sources, "Return all active triggers for this event") 
      .def("get_timestamp_gps48"      , &MasterTriggerEvent::get_timestamp_gps48, "48bit GPS timestamp") 
      .def("get_timestamp_abs48"      , &MasterTriggerEvent::get_timestamp_abs48, "Absolute 48bit timestamp") 
      .def_readonly("event_status"    , &MasterTriggerEvent::event_status, "MasterTriggerEvent event status field" ) 
      .def_readonly("event_id"        , &MasterTriggerEvent::event_id, "MTB event id" ) 
      .def_readonly("timestamp"       , &MasterTriggerEvent::timestamp                )
      .def_readonly("tiu_timestamp"   , &MasterTriggerEvent::tiu_timestamp            )
      .def_readonly("tiu_gps16"       , &MasterTriggerEvent::tiu_gps16                )
      .def_readonly("tiu_gps32"       , &MasterTriggerEvent::tiu_gps32                )
      .def_readonly("crc"             , &MasterTriggerEvent::crc                      )
      .def_readonly("trigger_source"  , &MasterTriggerEvent::trigger_source           )
      .def_readonly("dsi_j_mask"      , &MasterTriggerEvent::dsi_j_mask               )
      .def_readonly("channel_mask"    , &MasterTriggerEvent::channel_mask             )
      .def_readonly("mtb_link_mask"   , &MasterTriggerEvent::mtb_link_mask            )
      .def("__repr__",        [](const MasterTriggerEvent &ev) {
                                 return ev.to_string(); 
                                 }) 
    ;

    py::enum_<TriggerType>(m, "TriggerType")
      .value("Unknown",          TriggerType::Unknown  )
      .value("Gaps"   ,          TriggerType::Gaps     )
      .value("Any"    ,          TriggerType::Any      )
      .value("Track"  ,          TriggerType::Track    )
      .value("TrackCentral",     TriggerType::TrackCentral)
      .value("Poisson",          TriggerType::Poisson  )
      .value("Forced" ,          TriggerType::Forced   )
      ;
    
    py::enum_<LTBThreshold>(m, "LTBThreshold")
      .value("Unknown",          LTBThreshold::Unknown )
      .value("NoHit"  ,          LTBThreshold::NoHit   )
      .value("Hit"    ,          LTBThreshold::Hit     )
      .value("Beta"   ,          LTBThreshold::Beta    )
      .value("Veto"   ,          LTBThreshold::Veto    )
      ;

    py::enum_<PacketType>(m, "PacketType")
      .value("Unknown",          PacketType::Unknown   )
      .value("Command",          PacketType::Command   )
      .value("RBEvent",          PacketType::RBEvent   )
      .value("TofEvent",         PacketType::TofEvent  )
      .value("RBMoniData",       PacketType::RBMoni    )
      .value("PAMoniData",       PacketType::PAMoniData)
      .value("PBMoniData",       PacketType::PBMoniData)
      .value("LTBMoniData",      PacketType::LTBMoniData)
      .value("HeartBeat",        PacketType::HeartBeat     )
      .value("CPUMoniData",      PacketType::CPUMoniData   )
      .value("MasterTrigger",    PacketType::MasterTrigger )
      .value("RBCalibration",    PacketType::RBCalibration )
      .value("MtbMoniData",      PacketType::MTBMoni       )
      .value("TofEventSummary",  PacketType::TofEventSummary)
      .value("RBWaveform",       PacketType::RBWaveform     )
      ;
      //.export_values();

    py::enum_<PADDLE_END>(m, "PADDLE_END")
        .value("A", PADDLE_END::A)
        .value("B", PADDLE_END::B)
        .value("UNKNOWN", PADDLE_END::UNKNOWN)
        //.export_values();
        ;

    py::class_<MtbMoniData>(m, "MtbMoniData",
            "Monitoring data from the master trigger board.")
        .def(py::init())
        .def("from_bytestream"      , &MtbMoniData::from_bytestream)
        .def("from_tofpacket"       , &MtbMoniData::from_tofpacket)
        .def_readonly("fpga_temp"   , &MtbMoniData::fpga_temp   ) 
        .def_readonly("fpga_vccint" , &MtbMoniData::fpga_vccint ) 
        .def_readonly("fpga_vccaux" , &MtbMoniData::fpga_vccaux ) 
        .def_readonly("fpga_vccbram", &MtbMoniData::fpga_vccbram) 
        .def_readonly("rate"        , &MtbMoniData::rate        ) 
        .def_readonly("lost_rate"   , &MtbMoniData::lost_rate   ) 
        .def("__repr__",          [](const MtbMoniData &moni) {
                                  return moni.to_string();
                                  }) 
    ;
    
    py::class_<CPUMoniData>(m, "CPUMoniData",
            "Monitoring data from the tof flight computer (TOF-CPU)")
        .def(py::init())
        .def("from_bytestream"      , &CPUMoniData::from_bytestream)
        .def_readonly("uptime"      , &CPUMoniData::uptime     ) 
        .def_readonly("disk_usage"  , &CPUMoniData::disk_usage ) 
        .def_readonly("cpu_freq"    , &CPUMoniData::cpu_freq   ) 
        .def_readonly("cpu_temp"    , &CPUMoniData::cpu_temp   ) 
        .def_readonly("cpu0_temp"   , &CPUMoniData::cpu0_temp  ) 
        .def_readonly("cpu1_temp"   , &CPUMoniData::cpu1_temp  ) 
        .def_readonly("mb_temp"     , &CPUMoniData::mb_temp    ) 
        .def("__repr__",          [](const CPUMoniData &moni) {
                                  return moni.to_string();
                                  }) 
    ;

    py::class_<LTBMoniData>(m, "LTBMoniData",
            "Environmental sensors & thresholds for LocalTriggerBoards")
        .def(py::init())
        .def("from_bytestream",   &LTBMoniData::from_bytestream,
                "Factory function to recreate LTBMoniData from byte representation")
        .def_readonly("board_id", &LTBMoniData::board_id,
                "The ID of the RB the LTB is connected to")
        .def_readonly("trenz_temp", &LTBMoniData::trenz_temp) 
        .def_readonly("ltb_temp"  , &LTBMoniData::ltb_temp)
        .def_readonly("thresholds", &LTBMoniData::thresh,
                "Trigger thresholds applied to the low gain signal of paddle ends. In mV")
        .def("__repr__",          [](const LTBMoniData &moni) {
                                  return moni.to_string();
                                  }) 
    ;
    
    py::class_<PBMoniData>(m, "PBMoniData",
            "Sensors on the Powerboards")
        .def(py::init())
        .def("from_bytestream",   &PBMoniData::from_bytestream,
                "Factory function to recreate PBMoniData from byte representation")
        .def_readonly("board_id", &PBMoniData::board_id,
                "The ID of the RB the PB is connected to")
        .def_readonly("p3v6_preamp_vcp", &PBMoniData::p3v6_preamp_vcp  ) 
        .def_readonly("n1v6_preamp_vcp", &PBMoniData::n1v6_preamp_vcp ) 
        .def_readonly("p3v4f_ltb_vcp"  , &PBMoniData::p3v4f_ltb_vcp   ) 
        .def_readonly("p3v4d_ltb_vcp"  , &PBMoniData::p3v4d_ltb_vcp   ) 
        .def_readonly("p3v6_ltb_vcp"   , &PBMoniData::p3v6_ltb_vcp ) 
        .def_readonly("n1v6_ltb_vcp"   , &PBMoniData::n1v6_ltb_vcp ) 
        .def_readonly("pds_temp"       , &PBMoniData::pds_temp ) 
        .def_readonly("pas_temp"       , &PBMoniData::pas_temp ) 
        .def_readonly("nas_temp"       , &PBMoniData::nas_temp ) 
        .def_readonly("shv_temp"       , &PBMoniData::shv_temp ) 
        .def("__repr__",          [](const PBMoniData &moni) {
                                  return moni.to_string();
                                  }) 
    ;
    
    py::class_<PAMoniData>(m, "PAMoniData",
            "Sensors for the preamps")
        .def(py::init())
        .def("from_bytestream",   &PAMoniData::from_bytestream,
                "Factory function to recreate PAMoniData from byte representation")
        .def_readonly("board_id", &PAMoniData::board_id,
                "The ID of the RB which is used to read out these sensors")
        .def_readonly("temps",    &PAMoniData::temps  ) 
        .def_readonly("biases",   &PAMoniData::biases ) 
        .def("__repr__",          [](const PAMoniData &moni) {
                                  return moni.to_string();
                                  }) 
    ;

    py::class_<RBMoniData>(m, "RBMoniData",
            "Packet with monitoring data from the individual readout boards.")
        .def(py::init())
        .def("from_bytestream",          &RBMoniData::from_bytestream)
        .def_readonly("rate",            &RBMoniData::rate)
        .def_readonly("board_id",        &RBMoniData::board_id)           
        .def_readonly("tmp_drs",         &RBMoniData::tmp_drs)           
        .def_readonly("tmp_clk",         &RBMoniData::tmp_clk)           
        .def_readonly("tmp_adc",         &RBMoniData::tmp_adc)           
        .def_readonly("tmp_zynq",        &RBMoniData::tmp_zynq)           
        .def_readonly("tmp_lis3mdltr",   &RBMoniData::tmp_lis3mdltr)           
        .def_readonly("tmp_bm280",       &RBMoniData::tmp_bm280)           
        .def_readonly("pressure",        &RBMoniData::pressure)           
        .def_readonly("humidity",        &RBMoniData::humidity)           
        .def_readonly("mag_x",           &RBMoniData::mag_x)           
        .def_readonly("mag_y",           &RBMoniData::mag_y)           
        .def_readonly("mag_z",           &RBMoniData::mag_z)           
        .def_readonly("mag_tot",         &RBMoniData::mag_tot)           
        .def_readonly("drs_dvdd_voltage",&RBMoniData::drs_dvdd_voltage)           
        .def_readonly("drs_dvdd_current",&RBMoniData::drs_dvdd_current)           
        .def_readonly("drs_dvdd_power",  &RBMoniData::drs_dvdd_power)           
        .def_readonly("p3v3_voltage",    &RBMoniData::p3v3_voltage)           
        .def_readonly("p3v3_current",    &RBMoniData::p3v3_current)           
        .def_readonly("p3v3_power",      &RBMoniData::p3v3_power)           
        .def_readonly("zynq_voltage",    &RBMoniData::zynq_voltage)           
        .def_readonly("zynq_current",    &RBMoniData::zynq_current)           
        .def_readonly("zynq_power",      &RBMoniData::zynq_power)           
        .def_readonly("p3v5_voltage",    &RBMoniData::p3v5_voltage)           
        .def_readonly("p3v5_current",    &RBMoniData::p3v5_current)           
        .def_readonly("p3v5_power",      &RBMoniData::p3v5_power)           
        .def_readonly("adc_dvdd_voltage",&RBMoniData::adc_dvdd_voltage)           
        .def_readonly("adc_dvdd_current",&RBMoniData::adc_dvdd_current)           
        .def_readonly("adc_dvdd_power",  &RBMoniData::adc_dvdd_power)           
        .def_readonly("adc_avdd_voltage",&RBMoniData::adc_avdd_voltage)           
        .def_readonly("adc_avdd_current",&RBMoniData::adc_avdd_current)           
        .def_readonly("adc_avdd_power",  &RBMoniData::adc_avdd_power)           
        .def_readonly("drs_avdd_voltage",&RBMoniData::drs_avdd_voltage)          
        .def_readonly("drs_avdd_current",&RBMoniData::drs_avdd_current)           
        .def_readonly("drs_avdd_power",  &RBMoniData::drs_avdd_power)           
        .def_readonly("n1v5_voltage",    &RBMoniData::n1v5_voltage)                 
        .def_readonly("n1v5_current",    &RBMoniData::n1v5_current)                 
        .def_readonly("n1v5_power",      &RBMoniData::n1v5_power)                     
        .def("__repr__",          [](const RBMoniData &moni) {
                                  return rbmoni_to_string(moni);
                                  }) 
    ;
    
    py::class_<TofEventHeader>(m, "TofEventHeader",
        "Meta information, primary particle reconstruction & general variables.")
        .def(py::init())
        .def_readonly("run_id"              , &TofEventHeader::run_id             )
        .def_readonly("event_id"            , &TofEventHeader::event_id           ) 
        .def_readonly("timestamp32"         , &TofEventHeader::timestamp32        ) 
        .def_readonly("timestamp16"         , &TofEventHeader::timestamp16        ) 
        .def_readonly("primary_beta"        , &TofEventHeader::primary_beta       ) 
        .def_readonly("primary_beta_unc"    , &TofEventHeader::primary_beta_unc   ) 
        .def_readonly("primary_charge"      , &TofEventHeader::primary_charge     ) 
        .def_readonly("primary_charge_unc"  , &TofEventHeader::primary_charge_unc ) 
        .def_readonly("primary_outer_tof_x" , &TofEventHeader::primary_outer_tof_x) 
        .def_readonly("primary_outer_tof_y" , &TofEventHeader::primary_outer_tof_y) 
        .def_readonly("primary_outer_tof_z" , &TofEventHeader::primary_outer_tof_z) 
        .def_readonly("primary_inner_tof_x" , &TofEventHeader::primary_inner_tof_x) 
        .def_readonly("primary_inner_tof_y" , &TofEventHeader::primary_inner_tof_y) 
        .def_readonly("primary_inner_tof_z" , &TofEventHeader::primary_inner_tof_z)  
        .def_readonly("nhit_outer_tof"      , &TofEventHeader::nhit_outer_tof     ) 
        .def_readonly("nhit_inner_tof"      , &TofEventHeader::nhit_inner_tof     ) 
        .def_readonly("trigger_info"        , &TofEventHeader::trigger_info       ) 
        .def_readonly("ctr_etx"             , &TofEventHeader::ctr_etx            ) 
        .def_readonly("n_paddles"           , &TofEventHeader::n_paddles          ) 
        .def("get_timestamp48"              , &TofEventHeader::get_timestamp48    )
        .def("from_bytestream"              , &TofEvent::from_bytestream          )
        //.def("from_tofpacket"               ,&TofEvent::from_tofpacket)
        .def("__repr__",           [](const TofEventHeader &th) {
                                   return th.to_string(); 
                                   }) 

    ;

    
    py::class_<TofEvent>(m, "TofEvent")
        .def(py::init())
        .def_readonly("header"              ,&TofEvent::header,
                "Online reconstruction and summary information")
        .def_readonly("mt_event"            ,&TofEvent::mt_event,
                "The event information comming from the MasterTriggerBoard")
        .def_readonly("missing_hits"        ,&TofEvent::missing_hits)
        .def_readonly("rbevents"            ,&TofEvent::rb_events,
                "A list of all RBEvents which contributed to this event")
        .def("get_rbids"                    ,&TofEvent::get_rbids,
                "Get a list of all RB ids contributing to this event"
                                             )
        .def("get_rbevent"                  ,&TofEvent::get_rbevent,
                 "Return a the event for this specif RB id",
                                             py::arg("rb_id"))
        .def("from_bytestream"              ,&TofEvent::from_bytestream)
        .def("from_tofpacket"               ,&TofEvent::from_tofpacket,
                 "Factory function: Unpack a TofEvent from a TofPacket")
        .def("__repr__",           [](const TofEvent &te) {
                                   return te.to_string(); 
                                   }) 

    ;

    py::class_<TofPacket>(m, "TofPacket")
        .def(py::init())
        .def("from_bytestream",       &TofPacket::from_bytestream)
        .def_readonly("payload",      &TofPacket::payload)
        .def("unpack",                &TofPacket::unpack<TofEvent>)
        .def_readonly("payload_size", &TofPacket::payload_size)
        .def_readonly("packet_type",  &TofPacket::packet_type)
        .def("__repr__",          [](const TofPacket &pkg) {
                                  return pkg.to_string();
                                  }); 

    py::class_<TofHit>(m, "TofHit",
            "Reconstructed waveform information for the 2 channels of a paddle.")
        .def(py::init())
        .def("from_bytestream",        &TofHit::from_bytestream, 
               "Factory method to deserialize a TofHit")
        .def_readonly("paddle_id",     &TofHit::paddle_id     , 
               "The unique identifier for this paddle (1-160)")
        .def_property_readonly("time_a",        &TofHit::get_time_a  ,
               "Reconstructed peak start time for side A")
        .def_property_readonly("time_b",        &TofHit::get_time_b  ,
               "Reconstructed peak start time for side B")
        .def_property_readonly("peak_a",        &TofHit::get_peak_a  ,
               "Reconstructed peak height for side A")
        .def_property_readonly("peak_b",        &TofHit::get_peak_b  ,
               "Reconstructed peak height for side B")
        .def_property_readonly("charge_a",      &TofHit::get_charge_a,
               "Reconstructed charge for side A")
        .def_property_readonly("charge_b",      &TofHit::get_charge_b,
               "Reconstructed charge for side B")
        .def_property_readonly("charge_min_i",  &TofHit::get_charge_min_i,
               "Reconstructed paddle charge in units of MinI") 
        .def_property_readonly("x_pos",         &TofHit::get_x_pos,
               "Reconstructed position along the paddle")
        .def_property_readonly("t_avg",         &TofHit::get_t_avg,
               "(FIXME) - the reconstructed hit time") 
        .def_readonly("timestamp16",   &TofHit::timestamp16,
               "MSB part of the timestamp (slow)")
        .def_readonly("timestamp32",   &TofHit::timestamp32,
               "LSB part of the timestamp (fast)")
        .def_property_readonly("timestamp48",   &TofHit::get_timestamp48,
               "Complete timestamp (48 bits)")
        .def("__repr__",          [](const TofHit &th) {
                                  return th.to_string();
                                  }) 

    ;
    
   py::class_<RBCalibration>(m, "RBCalibration", 
      "RBCalibration holds th calibration constants (one per bin) per each channel for a single RB. This needs to be used in order to convert ADC to voltages/nanoseconds!")
       .def(py::init())
       .def_readonly("rb_id",      &RBCalibration::rb_id)
       .def_readonly("d_v",        &RBCalibration::d_v)
       .def_readonly("timestamp",  &RBCalibration::timestamp)
       .def_readonly("v_offsets",  &RBCalibration::v_offsets)
       .def_readonly("v_incs",     &RBCalibration::v_incs)
       .def_readonly("v_dips",     &RBCalibration::v_dips)
       .def_readonly("t_bin",      &RBCalibration::t_bin)
       .def_readonly("noi_data",   &RBCalibration::noi_data)
       .def_readonly("vcal_data",  &RBCalibration::vcal_data)
       .def_readonly("tcal_data",  &RBCalibration::tcal_data)
       .def_static("disable_eventdata",   &RBCalibration::disable_eventdata,
            "Don't load event data from a calibration file (if available). Just load the calibration constants. (This only works with binary files.")
       .def("from_txtfile" ,       &RBCalibration::from_txtfile,
            "Initialize the RBCalibration from a file with exactly one TofPacket")
       .def_static("from_file" ,          &RBCalibration::from_file,
            "Initialize the RBCalibration from a file with exactly one TofPacket of type RBCalibration",
            py::arg("filename"), py::arg("discard_events") = true)
       .def("from_tofpacket",      unpack_tp_to_rbcalibration,
            "Unpack a RBCalibration from a compatible tofpacket") 
       .def("nanoseconds",         wrap_rbcalibration_nanoseconds_allchan_rbevent,
            "Apply timing calibration to adc values of all channels")
       .def("voltages",         wrap_rbcalibration_voltages_allchan_rbevent,
            "Apply voltage calibration to adc values of all channels. Allows for spike cleaning (optional)",
            py::arg("event"), py::arg("spike_cleaning") = false)
       .def("nanoseconds",         wrap_rbcalibration_nanoseconds_rbevent,
            "Apply timing calibration to adc values of a specific channel",
            py::arg("event"), py::arg("channel"))
       .def("voltages",         wrap_rbcalibration_voltages_rbevent,
            "Apply voltage calibration to adc values of a specific channel",
            py::arg("event"), py::arg("channel"))
       .def("from_bytestream",  &RBCalibration::from_bytestream, 
            "Deserialize a RBCalibration object from a Vec<u8>")
       .def("__repr__",        [](const RBCalibration &cali) {
                                  return cali.to_string();
                                  }) 
   ;

   // I/O functions
   m.def("get_tofpackets", &wrap_get_tofpackets_from_stream,
           "Get TofPackets from list of bytes",
           py::arg("bytestream"), py::arg("pos"), py::arg("filter") = PacketType::Unknown);
   m.def("get_tofpackets", &wrap_get_tofpackets_from_file,
           "Get TofPackets from a file on disk",
           py::arg("filename"), py::arg("filter") = PacketType::Unknown);
   m.def("unpack_tofevents",              &wrap_unpack_tofevents_from_tofpackets_from_stream,
                                          "Get TofEvents directly from list of bytes but the bytes are encoded TofPackets..\nArgs:\n * bytestream [list of char]\n * pos - start at position in stream\n",
                                          py::arg("bytestream"), py::arg("pos"));
   m.def("unpack_tofevents",              &wrap_unpack_tofevents_from_tofpackets_from_file,
                                          "Get TofEvents from a file on disk containing TofPackets. In case the packets contain RBEvents, they will be unpacked automatically.\nArgs:\n * filename - full path to file on disk containing serialized TofPackets.",
                                          py::arg("filename"));
   m.def("get_event_ids_from_raw_stream", &get_event_ids_from_raw_stream);
   m.def("get_bytestream_from_file",      &get_bytestream_from_file);
   m.def("get_rbeventheaders",            &get_rbeventheaders); 
   

   // Calibration functions
   m.def("remove_spikes",            &remove_spikes_helper);
}
