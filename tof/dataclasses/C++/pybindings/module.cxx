#include <fstream>

#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/complex.h>
#include <pybind11/functional.h>
#include <pybind11/chrono.h>
#include <pybind11/numpy.h>

#include "packets/REventPacket.h"
#include "packets/RPaddlePacket.h"
#include "packets/tof_packet.h"
#include "packets/CommandPacket.h"
#include "packets/MasterTriggerPacket.h"
#include "packets/monitoring.h"
#include "events/tof_event_header.hpp"

#include "io.hpp"
#include "serialization.h"
#include "calibration.h"
#include "blobroutines.h"
#include "WaveGAPS.h"
#include "TOFCommon.h"
#include "events.h"

#include "tof_typedefs.h"

#include "helpers.hpp"

using namespace GAPS;
using namespace pybind11::literals;
namespace py = pybind11;

/********************/
// helpers

int static_helper(RPaddlePacket& pp){
    return RPaddlePacket::calculate_length();
}

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

Vec<u16> ch_head_getter(BlobEvt_t evt)
{
    Vec<u16> ch_head;
    for (size_t k=0; k<NCHN; k++) 
    {ch_head.push_back(evt.ch_head[k]);}
    return ch_head;
}

Vec<u64> ch_trail_getter(BlobEvt_t evt)
{
    Vec<u64> ch_trail;
    for (size_t k=0; k<NCHN; k++) 
    {ch_trail.push_back(evt.ch_trail[k]);}
    return ch_trail;
}

Vec<Vec<i16>> ch_getter(BlobEvt_t evt)
{
    Vec<Vec<i16>> channels;
    for (size_t k=0; k<NCHN; k++) 
      {  channels.push_back({});
         for (size_t l=0; l < NWORDS; l++)
            {
               channels[k].push_back(evt.ch_adc[k][l]);
            }
      }
    return channels;
}

usize get_current_blobevent_size() {
  return 36 + (NCHN*2) + (NCHN*NWORDS*2) + (NCHN*4) + 8;
}

//bytestream blobevent_encoder(BlobEvt_t evt, size_t startpos)
//{
//  bytestream buffer;
//  buffer.reserve(get_current_blobevent_size());
//  for (size_t k=0; k<get_current_blobevent_size(); k++)
//  {buffer.push_back(0);}
//  encode_blobevent(&evt, buffer, startpos);
//  return buffer;
//}
//
//BlobEvt_t blobevent_decoder(bytestream buffer, size_t startpos)
//{
//  BlobEvt_t evt = decode_blobevent(buffer, startpos);
//  return evt;
//}

std::string BlobEvtToString(BlobEvt_t event)
{
   std::string output = "";
   output += "head "      + std::to_string(event.head )      + "\n" ;
   output += "status "    + std::to_string(event.status )    + "\n" ;
   output += "len "       + std::to_string(event.len )       + "\n" ;
   output += "roi "       + std::to_string(event.roi )       + "\n" ;
   output += "dna "       + std::to_string(event.dna )       + "\n" ;
   output += "fw_hash "   + std::to_string(event.fw_hash )   + "\n" ;
   output += "id "        + std::to_string(event.id )        + "\n" ;
   output += "ch_mask "   + std::to_string(event.ch_mask )   + "\n" ;
   output += "event_ctr " + std::to_string(event.event_ctr ) + "\n" ;
   output += "dtap0 "     + std::to_string(event.dtap0 )     + "\n" ;
   output += "dtap1 "     + std::to_string(event.dtap1 )     + "\n" ;
   output += "timestamp " + std::to_string(event.timestamp ) + "\n" ;
   output += "stop_cell " + std::to_string(event.stop_cell ) + "\n" ;
   output += "crc32 "     + std::to_string(event.crc32 )     + "\n" ;
   output += "tail "      + std::to_string(event.tail)       ;
   return output;
}

template<class T>
void nullsetter(T foo) 
{
    std::cerr << "Can not set this property!" << std::endl;
}

void set_payload_helper(TofPacket &packet,
                        const Vec<u8> payload) {
  packet.payload = payload;
  packet.payload_size = payload.size();
}

//void set_ptype_helper(TofPacket &packet,
//                      const PacketType &ptype) {
//  packet.packet_type = ptype;
//}

/********************/

BlobEvt_t read_event_helper(std::string filename, i32 n)
{
    FILE* f = fopen(filename.c_str(), "rb");
    BlobEvt_t event;
    while(n >= 0) {
      ReadEvent(f, &event, false);
      n--;
    }
    return event;
}

/********************/

/*****************
 * Dismantle a readoutboard file and return the individual
 * fields as arrays in a python dictionary
 *
 */
py::dict splice_readoutboard_datafile(const std::string filename) {
  bytestream stream             = get_bytestream_from_file(filename);
  std::vector<BlobEvt_t> events = get_events_from_stream(stream, 0);
  Vec<u32> event_ids  = Vec<u32>(); 
  Vec<u16> stop_cells = Vec<u16>(); 
  Vec<u64> timestamps = Vec<u64>();
  
  // channels, times
  Vec<Vec<u16>> t_1     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_2     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_3     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_4     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_5     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_6     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_7     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_8     = Vec<Vec<u16>>();
  Vec<Vec<u16>> t_9     = Vec<Vec<u16>>();
  
  Vec<Vec<i16>> adc_1     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_2     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_3     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_4     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_5     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_6     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_7     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_8     = Vec<Vec<i16>>();
  Vec<Vec<i16>> adc_9     = Vec<Vec<i16>>();
 
  for (auto ev : events) {
     event_ids .push_back(ev.event_ctr);
     stop_cells.push_back(ev.stop_cell);
     timestamps.push_back(ev.timestamp);
     adc_1       .push_back(Vec<i16>(ev.ch_adc[0], std::end(ev.ch_adc[0])));
     adc_2       .push_back(Vec<i16>(ev.ch_adc[1], std::end(ev.ch_adc[1])));
     adc_3       .push_back(Vec<i16>(ev.ch_adc[2], std::end(ev.ch_adc[2])));
     adc_4       .push_back(Vec<i16>(ev.ch_adc[3], std::end(ev.ch_adc[3])));
     adc_5       .push_back(Vec<i16>(ev.ch_adc[4], std::end(ev.ch_adc[4])));
     adc_6       .push_back(Vec<i16>(ev.ch_adc[5], std::end(ev.ch_adc[5])));
     adc_7       .push_back(Vec<i16>(ev.ch_adc[6], std::end(ev.ch_adc[6])));
     adc_8       .push_back(Vec<i16>(ev.ch_adc[7], std::end(ev.ch_adc[7])));
     adc_9       .push_back(Vec<i16>(ev.ch_adc[8], std::end(ev.ch_adc[8])));
  }
  py::dict data(
                "event_id"_a  =py::array_t<u32>(event_ids.size(),  event_ids.data()),\
                "stop_cell"_a =py::array_t<u16>(stop_cells.size(), stop_cells.data()),\
                "timestamps"_a=py::array_t<u64>(timestamps.size(), timestamps.data()),\
                "adc_ch1"_a=adc_1,\
                "adc_ch2"_a=adc_2,\
                "adc_ch3"_a=adc_3,\
                "adc_ch4"_a=adc_4,\
                "adc_ch5"_a=adc_5,\
                "adc_ch6"_a=adc_6,\
                "adc_ch7"_a=adc_7,\
                "adc_ch8"_a=adc_8,\
                "adc_ch9"_a=adc_9);
  return data;
}


int get_nevents_from_file(std::string filename){
  FILE* f = fopen(filename.c_str(), "rb");
  BlobEvt_t event;
  i32 result = 0;
  u32 nevents = 0;
  while (result >= 0) {
    result = ReadEvent(f, &event, false);
    nevents++;
  }
  return nevents;
}

/********************/

std::vector<Calibrations_t> read_calibration_file (std::string filename) {
  std::vector<Calibrations_t> all_channel_calibrations = std::vector<Calibrations_t>{NCHN};
  std::fstream calfile(filename.c_str(), std::ios_base::in);
  if (calfile.fail()) {
    std::cerr << "[ERROR] Can't open " << filename << " - not calibrating" << std::endl;
    return all_channel_calibrations;
  }
  for (size_t i=0; i<NCHN; i++) {
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vofs[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vdip[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vinc[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].tbin[j];
  }
  return all_channel_calibrations;
}

/********************/

Vec<Vec<f64>> offset_getter(const std::vector<Calibrations_t> &cal)
{
  Vec<Vec<f64>> offsets;
  for (size_t k=0; k<NCHN; k++) 
    {  offsets.push_back({});
       for (size_t l=0; l < NWORDS; l++)
          {
             offsets[k].push_back(cal[k].vofs[l]);
          }
    }
  return offsets;
}

Vec<Vec<f64>> dip_getter(const std::vector<Calibrations_t> &cal)
{
    Vec<Vec<f64>> dips;
    for (size_t k=0; k<NCHN; k++) 
      {  dips.push_back({});
         for (size_t l=0; l < NWORDS; l++)
           {
             dips[k].push_back(cal[k].vdip[l]);
           }
      }
    return dips;
}

Vec<Vec<f64>> increment_getter(const std::vector<Calibrations_t> &cal)
{
  Vec<Vec<f64>> incs;
  for (size_t k=0; k<NCHN; k++) 
    {  incs.push_back({});
       for (uint l=0; l < NWORDS; l++)
        {
          incs[k].push_back(cal[k].vinc[l]);
        }
    }
  return incs;
}

Vec<Vec<f64>> tbin_getter(const std::vector<Calibrations_t> cal)
{
    Vec<Vec<f64>> tbins;
    for (size_t k=0; k<NCHN; k++) 
      {  tbins.push_back({});
         for (size_t l=0; l < NWORDS; l++)
            {
              tbins[k].push_back(cal[k].tbin[l]);
            }
      }
    return tbins;
}

/********************/

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
    m.doc() = "GAPS Tof dataclasses and utility tools";
    
    py::class_<Gaps::TofPacketReader>(m, "TofPacketReader") 
      .def(py::init<String>())  
      .def("get_next_packet", &Gaps::TofPacketReader::get_next_packet,
                              "iterate over the packets and get the next packet from the file")
      .def("__repr__",        [](const Gaps::TofPacketReader &reader) {
                                  return "<TofPacketReader : "
                                  + reader.get_filename() + ">";
                                  }) 
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
      .export_values();

    py::class_<RBEventHeader>(m, "RBEventHeader", "The event header contains the event id, information about active channels, temperatures, trigger stop cells etc. Basically everythin except channel adc data.")
      .def(py::init())
      .def("from_bytestream"             , &RBEventHeader::from_bytestream, "Deserialize from a list of bytes")
      .def("extract_from_rbmemoryview"   , &RBEventHeader::extract_from_rbbinarydump, "Get header from full rbevent binary stream ('blob')")
      .def("get_active_data_channels"    , &RBEventHeader::get_active_data_channels, "Get a list of active channels, excluding ch9. Channel9 will (usually) always be on, as long as a single data channel is switched on as well.")
      .def("get_fpga_temp"               , &RBEventHeader::get_fpga_temp, "The FPGA temperature in C")
      .def("get_drs_temp"                , &RBEventHeader::get_drs_temp, "The DRS4 temperature in C, read out by software")
      .def("get_clock_cycles_48bit"      , &RBEventHeader::get_clock_cycles_48bit, "The complete 48bit timestamp, derived from the RB clock (usually 33MHz)")
      .def("get_n_datachan"              , &RBEventHeader::get_n_datachan)
      //.def("get_timestamp_16_corrected",   &RBEventHeader::get_timestamp_16_corrected)
      .def_readonly("channel_mask"       , &RBEventHeader::channel_mask)   
      .def_readonly("stop_cell"          , &RBEventHeader::stop_cell   )   
      .def_readonly("crc32"              , &RBEventHeader::crc32       )   
      .def_readonly("dtap0"              , &RBEventHeader::dtap0       )   
      .def_readonly("drs4_temp"          , &RBEventHeader::drs4_temp   )   
      .def_readonly("is_locked"          , &RBEventHeader::is_locked   )   
      .def_readonly("is_locked_last_sec" , &RBEventHeader::is_locked_last_sec)   
      .def_readonly("lost_trigger"       , &RBEventHeader::lost_trigger)   
      .def_readonly("event_fragment"     , &RBEventHeader::event_fragment)   
      .def_readonly("fpga_temp"          , &RBEventHeader::fpga_temp   )   
      .def_readonly("event_id"           , &RBEventHeader::event_id    )   
      .def_readonly("rb_id"              , &RBEventHeader::rb_id       )   
      //.def_readonly("timestamp_32"       , &RBEventHeader::timestamp_32)   
      //.def_readonly("timestamp_16"       , &RBEventHeader::timestamp_16)   
      .def_readonly("broken"             , &RBEventHeader::broken      )   
      .def("__repr__",        [](const RBEventHeader &h) {
                                   return h.to_string();
                                 })
    
      
    ;

    py::class_<RBEvent>(m, "RBEvent", "RBEvent contains an event header for this specific board as well as a (flexible) number of adc channels")
        .def(py::init())
        .def_readonly("header"              ,&RBEvent::header)
        .def_readonly("nchan"               ,&RBEvent::nchan)
        .def_readonly("npaddles"            ,&RBEvent::npaddles)
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



    py::class_<MasterTriggerEvent>(m, "MasterTriggerEvent", "The MasterTriggerEvent contains the information from the MTB.")
      .def(py::init())
      .def("from_bytestream", &MasterTriggerEvent::from_bytestream, "Deserialize from a list of bytes")
      .def_readonly("event_id"        , &MasterTriggerEvent::event_id, "MTB event id" ) 
      .def_readonly("timestamp"       , &MasterTriggerEvent::timestamp                )
      .def_readonly("tiu_timestamp"   , &MasterTriggerEvent::tiu_timestamp            )
      .def_readonly("gps_timestamp_16", &MasterTriggerEvent::tiu_gps_16               )
      .def_readonly("gps_timestamp_32", &MasterTriggerEvent::tiu_gps_32               )
      .def_readonly("board_mask"      , &MasterTriggerEvent::board_mask               )
      .def_readonly("n_paddles"       , &MasterTriggerEvent::n_paddles                ) 
      .def("__repr__",        [](const MasterTriggerEvent &ev) {
                                 return ev.to_string(); 
                                 }) 
    ;
  

    py::class_<CommandPacket>(m, "CommandPacket") 
      .def(py::init<TofCommand const&, u32 const>())  
      .def("to_bytestream",   &CommandPacket::to_bytestream  , "Translate the command to a list of bytes")
      .def("from_bytestream", &CommandPacket::from_bytestream, "Retrieve a command from a list of bytes")
      .def("get_command" ,    [](const CommandPacket &pk) {
                                  return pk.command;
                              })
      .def("__repr__",        [](const CommandPacket &pk) {
                                  return "<CommandPacket : "
                                  + tof_command_to_str(pk.command)
                                  + " "
                                  + std::to_string(pk.value) + ">";
                                  }) 
    ;

    py::enum_<TofResponse>(m, "TofResponse")
      .value("Success"                 ,TofResponse::Success) 
      .value("GeneralFailure"          ,TofResponse::GeneralFailure) 
      .value("EventNotReady"           ,TofResponse::EventNotReady) 
      .value("EventSerializationIssue" ,TofResponse::SerializationIssue) 
      .value("Unknown"                 ,TofResponse::Unknown) 
      .export_values()
    ;
   
    py::class_<ResponsePacket>(m, "ResponsePacket") 
      .def(py::init<TofResponse const&, u32 const>())  
      .def("to_bytestream",   &ResponsePacket::to_bytestream)
      .def("from_bytestream", &ResponsePacket::from_bytestream)
      .def("translate_response_code", &ResponsePacket::translate_response_code,
                                      "Translate the response code into some human readable string")
      .def("get_response"   ,    [](const ResponsePacket &pk) {
                                  return pk.response; 
                                 }
                              , "Get the RESPONSE_CODE from the response. This will provide further information.")
      .def("__repr__",        [](const ResponsePacket &pk) {
                                  return "<ResponsePacket : "
                                  + tof_response_to_str(pk.response)
                                  + " "
                                  + pk.translate_response_code(pk.value) + ">";
                                  }) 
    ;
    py::enum_<PacketType>(m, "PacketType")
      .value("Unknown",   PacketType::Unknown   )
      .value("Command",   PacketType::Command   )
      .value("PT_RBEvent",   PacketType::RBEvent   )
      .value("PT_TofEvent",  PacketType::TofEvent  )
      .value("Monitor",   PacketType::Monitor   )
      .value("Scalar",    PacketType::Scalar    )
      .value("HeartBeat", PacketType::HeartBeat )
      .value("MasterTrigger", PacketType::MasterTrigger )
      .export_values();

    py::enum_<PADDLE_END>(m, "PADDLE_END")
        .value("A", PADDLE_END::A)
        .value("B", PADDLE_END::B)
        .value("UNKNOWN", PADDLE_END::UNKNOWN)
        .export_values();
    
    py::class_<MtbMoniData>(m, "MtbMoniData",
            "Monitoring data from the master trigger board.")
        .def(py::init())
        .def("from_bytestream"      , &MtbMoniData::from_bytestream)
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
    py::class_<TofCmpMoniData>(m, "TofCmpMoniData",
            "Monitoring data from the tof flight computer (TOF-CPU)")
        .def(py::init())
        .def("from_bytestream"    , &TofCmpMoniData::from_bytestream)
        .def_readonly("core1_tmp" , &TofCmpMoniData::core1_tmp ) 
        .def_readonly("core2_tmp" , &TofCmpMoniData::core2_tmp) 
        .def_readonly("pch_tmp"   , &TofCmpMoniData::pch_tmp  ) 
        .def("__repr__",          [](const TofCmpMoniData &moni) {
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
        .def_readonly("drs_avdd_voltage",&RBMoniData::adc_avdd_power)           
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
        .def_readonly("timestamp_32"        , &TofEventHeader::timestamp_32       ) 
        .def_readonly("timestamp_16"        , &TofEventHeader::timestamp_16       ) 
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
        .def("from_bytestream"              , &TofEvent::from_bytestream          )
        //.def("from_tofpacket"               ,&TofEvent::from_tofpacket)
        .def("__repr__",           [](const TofEventHeader &th) {
                                   return th.to_string(); 
                                   }) 

    ;

    
    py::class_<TofEvent>(m, "TofEvent")
        .def(py::init())
        //.def_readonly("header"              ,&TofEvent::header)
        .def_readonly("mt_event"            ,&TofEvent::mt_event)
        .def_readonly("missing_hits"        ,&TofEvent::missing_hits)
        .def_readonly("rbevents"            ,&TofEvent::rb_events)
        .def("get_rbids"                    ,&TofEvent::get_rbids,
                                             "Get a list of all RB ids contributing to this event."
                                             )
        .def("get_rbevent"                  ,&TofEvent::get_rbevent,
                                             "Return a the event for this specif RB id",
                                             py::arg("rb_id"))
        .def("from_bytestream"              ,&TofEvent::from_bytestream)
        .def("from_tofpacket"               ,&TofEvent::from_tofpacket)
        .def("__repr__",           [](const TofEvent &te) {
                                   return te.to_string(); 
                                   }) 

    ;

    py::class_<REventPacket>(m, "REventPacket")
        .def(py::init())
        .def("to_bytestream"                ,&REventPacket::serialize)
        .def("from_bytestream"              ,&REventPacket::deserialize)
        .def("calculate_length"             ,&REventPacket::calculate_length)
        .def("is_broken"                    ,&REventPacket::is_broken)
        .def("reset"                        ,&REventPacket::reset)
        .def("add_paddle_packet"            ,&REventPacket::add_paddle_packet)
        .def("is_broken"                    ,&REventPacket::is_broken)
        .def_readwrite("event_id"           ,&REventPacket::event_ctr)
        .def_readwrite("n_paddles"          ,&REventPacket::n_paddles)
        .def_readwrite("timestamp_32"       ,&REventPacket::timestamp_32)
        .def_readwrite("timestamp_16"       ,&REventPacket::timestamp_16)
        .def_readwrite("primary_beta"       ,&REventPacket::primary_beta)
        .def_readwrite("primary_beta_unc"   ,&REventPacket::primary_beta_unc)
        .def_readwrite("primary_charge"     ,&REventPacket::primary_charge)
        .def_readwrite("primary_charge_unc" ,&REventPacket::primary_charge_unc)
        .def_readwrite("primary_outer_tof_x",&REventPacket::primary_outer_tof_x)
        .def_readwrite("primary_outer_tof_y",&REventPacket::primary_outer_tof_y)
        .def_readwrite("primary_outer_tof_z",&REventPacket::primary_outer_tof_z)
        .def_readwrite("primary_inner_tof_x",&REventPacket::primary_inner_tof_x)
        .def_readwrite("primary_inner_tof_y",&REventPacket::primary_inner_tof_y)
        .def_readwrite("primary_inner_tof_z",&REventPacket::primary_inner_tof_z)
    .def_readonly("paddle_packets",      &REventPacket::paddle_info)
    .def("__repr__",          [](const REventPacket &ev) {
                                  return "<REventPacket : " + ev.to_string(true) + "'>";
                                  }) 

    ;
    py::class_<TofPacket>(m, "TofPacket")
        .def(py::init())
        .def("to_bytestream",         &TofPacket::to_bytestream)
        .def("from_bytestream",       &TofPacket::from_bytestream)
        .def("set_payload",           &set_payload_helper)
        //.def("set_packet_type",       &set_ptype_helper) 
        .def_readonly("head",         &TofPacket::head)
        .def_readonly("tail",         &TofPacket::tail)
        .def_readonly("payload",      &TofPacket::payload)
        .def_readonly("payload_size", &TofPacket::payload_size)
        .def_readonly("packet_type",  &TofPacket::packet_type)
        .def("__repr__",          [](const TofPacket &pkg) {
                                  return pkg.to_string();
                                  }); 

    py::class_<RPaddlePacket>(m, "RPaddlePacket")
        .def(py::init())
        .def("serialize",         &RPaddlePacket::serialize)
        .def("deserialize",       &RPaddlePacket::deserialize)
        .def("calculate_length",  &static_helper)
        .def("reset",             &RPaddlePacket::reset)
        .def("is_broken",         &RPaddlePacket::is_broken)
        .def("get_paddle_id",     &RPaddlePacket::get_paddle_id) 
        .def_property("time_a",   &RPaddlePacket::get_time_a, &RPaddlePacket::set_time_a)
        .def_property("time_b",   &RPaddlePacket::get_time_b, &RPaddlePacket::set_time_b)
        .def_property("peak_a",   &RPaddlePacket::get_peak_a, &RPaddlePacket::set_peak_a)
        .def_property("peak_b",   &RPaddlePacket::get_peak_b, &RPaddlePacket::set_peak_b)
        .def_property("charge_a", &RPaddlePacket::get_charge_a, &RPaddlePacket::set_charge_a)
        .def_property("charge_b", &RPaddlePacket::get_charge_b, &RPaddlePacket::set_charge_b)
        .def_property("charge_min_i",  &RPaddlePacket::get_charge_min_i, &RPaddlePacket::set_charge_min_i) 
        .def_property("x_pos",         &RPaddlePacket::get_x_pos, &RPaddlePacket::set_x_pos) 
        .def_property("t_avg",         &RPaddlePacket::get_t_avg, &RPaddlePacket::set_t_avg) 
        .def("get_time_a",        &RPaddlePacket::get_time_a) 
        .def("get_time_b",        &RPaddlePacket::get_time_b) 
        .def("get_peak_a",        &RPaddlePacket::get_peak_a) 
        .def("get_peak_b",        &RPaddlePacket::get_peak_b) 
        .def("get_charge_a",      &RPaddlePacket::get_charge_a) 
        .def("get_charge_b",      &RPaddlePacket::get_charge_b) 
        .def("get_charge_min_i",  &RPaddlePacket::get_charge_min_i) 
        .def("get_x_pos",         &RPaddlePacket::get_x_pos) 
        .def("get_t_avg",         &RPaddlePacket::get_t_avg) 
        //.def("set_paddle_id",     &RPaddlePacket::set_paddle_id) 
        .def("set_time_a",        &RPaddlePacket::set_time_a) 
        .def("set_time_b",        &RPaddlePacket::set_time_b) 
        .def("set_peak_a",        &RPaddlePacket::set_peak_a) 
        .def("set_peak_b",        &RPaddlePacket::set_peak_b) 
        .def("set_charge_a",      &RPaddlePacket::set_charge_a) 
        .def("set_charge_b",      &RPaddlePacket::set_charge_b) 
        .def("set_charge_min_i",  &RPaddlePacket::set_charge_min_i) 
        .def("set_x_pos",         &RPaddlePacket::set_x_pos) 
        .def("set_t_avg",         &RPaddlePacket::set_t_avg) 

        // atributes
        .def_readwrite("paddle_id" ,    &RPaddlePacket::paddle_id)
        .def_readwrite("timestamp_32" ,    &RPaddlePacket::timestamp_32)
        .def_readwrite("timestamp_16" ,    &RPaddlePacket::timestamp_16)
        //.def_readwrite("time_a" ,       &RPaddlePacket::time_a)
        //.def_readwrite("time_b" ,       &RPaddlePacket::time_b)
        //.def_readwrite("charge_a" ,     &RPaddlePacket::charge_a)
        //.def_readwrite("charge_b" ,     &RPaddlePacket::charge_b)
        //.def_readwrite("charge_min_i" , &RPaddlePacket::charge_min_i)
        //.def_readwrite("x_pos" ,        &RPaddlePacket::x_pos)
        //.def_readwrite("t_avg" ,        &RPaddlePacket::t_average)

        .def("__repr__",          [](const RPaddlePacket &pp) {
                                  return "<RPaddlePacket : " + pp.to_string() + "'>";
                                  }) 

    ;

    py::class_<RBEventMemoryView>(m, "RBEventMemoryView",
            "The RBEventMemoryView (formerly 'BlobEvent') is the direct representation of an event as read out by the readoutboard and layout in its RAM memory.")
       .def(py::init())
       .def_readonly("status"                  ,&RBEventMemoryView::status )
       .def_readonly("len"                     ,&RBEventMemoryView::len )
       .def_readonly("roi"                     ,&RBEventMemoryView::roi )
       .def_readonly("dna"                     ,&RBEventMemoryView::dna )
       .def_readonly("fw_hash"                 ,&RBEventMemoryView::fw_hash )
       .def_readonly("id"                      ,&RBEventMemoryView::id )
       .def_readonly("ch_mask"                 ,&RBEventMemoryView::ch_mask )
       .def_readonly("event_ctr"               ,&RBEventMemoryView::event_ctr )
       .def_readonly("dtap0"                   ,&RBEventMemoryView::dtap0 )
       .def_readonly("dtap1"                   ,&RBEventMemoryView::dtap1 )
       .def_readonly("timestamp"               ,&RBEventMemoryView::timestamp )
       .def("get_channel_adc"                  ,&RBEventMemoryView::get_channel_adc)
       .def_readonly("stop_cell"               ,&RBEventMemoryView::stop_cell )
       .def_readonly("crc32"                   ,&RBEventMemoryView::crc32 )
       .def("__repr__",  [] (const RBEventMemoryView &event) { 
         return rbeventmemoryview_to_string(event);
       })
   ;


    py::class_<BlobEvt_t>(m, "BlobEvt")
       .def(py::init())
       .def_readwrite("head"                    ,&BlobEvt_t::head ) 
       .def_readwrite("status"                  ,&BlobEvt_t::status )
       .def_readwrite("len"                     ,&BlobEvt_t::len )
       .def_readwrite("roi"                     ,&BlobEvt_t::roi )
       .def_readwrite("dna"                     ,&BlobEvt_t::dna )
       .def_readwrite("fw_hash"                 ,&BlobEvt_t::fw_hash )
       .def_readwrite("id"                      ,&BlobEvt_t::id )
       .def_readwrite("ch_mask"                 ,&BlobEvt_t::ch_mask )
       .def_readwrite("event_ctr"               ,&BlobEvt_t::event_ctr )
       .def_readwrite("dtap0"                   ,&BlobEvt_t::dtap0 )
       .def_readwrite("dtap1"                   ,&BlobEvt_t::dtap1 )
       .def_readwrite("timestamp"               ,&BlobEvt_t::timestamp )
       //.def_readwrite("timestamp_32"            ,&BlobEvt_t::timestamp_32 )
       //.def_readwrite("timestamp_16"            ,&BlobEvt_t::timestamp_16 )
       //.def_property("ch_head"      , &ch_head_getter , &nullsetter<std::vector<unsigned short>  )
       .def("get_ch_head"                       ,&ch_head_getter)
       .def("get_ch_adc"                        ,&ch_getter )
       .def("get_ch_trail"                      ,&ch_trail_getter )
       .def_readwrite("stop_cell"               ,&BlobEvt_t::stop_cell )
       .def_readwrite("crc32"                   ,&BlobEvt_t::crc32 )
       .def_readwrite("tail"                    ,&BlobEvt_t::tail)
       .def("__repr__",  [] (const BlobEvt_t &event) { 
               return "<BlobEvt_t \n" + BlobEvtToString(event) + ">";
               })
   ;
    
   py::class_<Waveform>(m, "Waveform")
       .def(py::init<int>())
       .def("SetWave"             , &Waveform::SetWave            )
       .def("SetTime"             , &Waveform::SetTime            )
       .def("SetThreshold"        , &Waveform::SetThreshold       )
       .def("GetWaveSize"         , &Waveform::GetWaveSize        ) 
       .def("SetBin"              , &Waveform::SetBin             )
       .def("GetBin"              , &Waveform::GetBin             )
       .def("GetBinTime"          , &Waveform::GetBinTime         )

       // this is simply  not defined
       //.def("GetBinDC"            , &Waveform::GetBinDC           )
       .def("GetMaxBin"           , &Waveform::GetMaxBin          )
       .def("GetMaxBinTime"       , &Waveform::GetMaxBinTime      )
       .def("GetMaxVal"           , &Waveform::GetMaxVal          )
       .def("GetMinBin"           , &Waveform::GetMinBin          )
       .def("GetMinBinTime"       , &Waveform::GetMinBinTime      )
       .def("GetMinVal"           , &Waveform::GetMinVal          )
       .def("GetPeakValue"        , &Waveform::GetPeakValue       )
       .def("Rescale"             , &Waveform::Rescale            ) 
       .def("Integrate"           , &Waveform::Integrate          )
       .def("SetPedestal"         , &Waveform::SetPedestal        )
       .def("SetRunPedestal"      , &Waveform::SetRunPedestal     )
       .def("SetPedRange"         , &Waveform::SetPedRange        )
       .def("SetPedBegin"         , &Waveform::SetPedBegin        )
       .def("GetPedRange"         , &Waveform::GetPedRange        )
       .def("GetPedBegin"         , &Waveform::GetPedBegin        )
       .def("GetPedestal"         , &Waveform::GetPedestal        )
       .def("GetPedsigma"         , &Waveform::GetPedsigma        )
       .def("CalcPedestalRange"   , &Waveform::CalcPedestalRange  )
       .def("CalcPedestalDynamic" , &Waveform::CalcPedestalDynamic)
       .def("SubtractPedestal"    , &Waveform::SubtractPedestal   )
       .def("SetMaxPeaks"         , &Waveform::SetMaxPeaks        )
       .def("GetMaxPeaks"         , &Waveform::GetMaxPeaks        )
       .def("CleanUpPeaks"        , &Waveform::CleanUpPeaks       )
       .def("GetNumPeaks"         , &Waveform::GetNumPeaks        )
       .def("SetCFDSFraction"     , &Waveform::SetCFDSFraction    )
       .def("SetCFDEFraction"     , &Waveform::SetCFDEFraction    )
       .def("SetCFDEOffset"       , &Waveform::SetCFDEOffset      )
       .def("FindPeaks"           , &Waveform::FindPeaks          )
       .def("FindTdc"             , &Waveform::FindTdc            )
       .def("GetSpikes"           , &Waveform::GetSpikes          )
       .def("GetTdcs"             , &Waveform::GetTdcs            )
       .def("GetCharge"           , &Waveform::GetCharge          )
       .def("GetHeight"           , &Waveform::GetHeight          )
       .def("GetWidth"            , &Waveform::GetWidth           )
       .def("GetPulsepars"        , &Waveform::GetPulsepars       ) 
       .def("GetPulsechi2"        , &Waveform::GetPulsechi2       ) 
       .def("GetNDF"              , &Waveform::GetNDF             ) 
       .def("FitPulse"            , &Waveform::FitPulse           )
       .def("GetNsPerBin"         , &Waveform::GetNsPerBin        ) 
       .def("GetOffset"           , &Waveform::GetOffset          ) 
       .def("GetTimingCorr"       , &Waveform::GetTimingCorr      )
       .def("GetImpedance"        , &Waveform::GetImpedance       ) 
       .def("SetImpedance"        , &Waveform::SetImpedance       )
       .def("__repr__",  [] (const Waveform &waveform) { 
               return "<GAPSWaveform>";
               })
   ;
   
   py::class_<Calibrations_t>(m, "Calibrations")
       .def(py::init())
   ;
  
   py::class_<RBCalibration>(m, "RBCalibration", 
      "RBCalibration holds th calibration constants (one per bin) per each channel for a single RB. This needs to be used in order to convert ADC to voltages/nanoseconds!")
       .def(py::init())
       .def_readonly("rb_id",      &RBCalibration::rb_id)
       .def_readonly("d_v",        &RBCalibration::d_v)
       .def_readonly("v_offsets",  &RBCalibration::v_offsets)
       .def_readonly("v_incs",     &RBCalibration::v_incs)
       .def_readonly("v_dips",     &RBCalibration::v_dips)
       .def_readonly("t_bin",      &RBCalibration::t_bin)
       .def_static("disable_eventdata",   &RBCalibration::disable_eventdata,
            "Don't load event data from a calibration file (if available). Just load the calibration constants. (This only works with binary files.")
       .def("from_tofpacket",      unpack_tp_to_rbcalibration,
            "Unpack a RBCalibration from a compatible tofpacket") 
       .def("nanoseconds",         wrap_rbcalibration_nanoseconds_allchan_rbevent,
            "Apply timing calibration to adc values of all channels")
       .def("nanoseconds",         wrap_rbcalibration_nanoseconds_allchan_rbeventmemoryview,
            "Apply timing calibration to adc values of all channels")
       .def("voltages",         wrap_rbcalibration_voltages_allchan_rbevent,
            "Apply voltage calibration to adc values of all channels. Allows for spike cleaning (optional)",
            py::arg("event"), py::arg("spike_cleaning") = false)
       .def("voltages",         wrap_rbcalibration_voltages_allchan_rbeventmemoryview,
            "Apply voltage calibration to adc values of all channels. Allows for spike cleaning (optional)",
            py::arg("event"), py::arg("spike_cleaning") = false)
       .def("nanoseconds",         wrap_rbcalibration_nanoseconds_rbevent,
            "Apply timing calibration to adc values of a specific channel",
            py::arg("event"), py::arg("channel"))
       .def("nanoseconds",         wrap_rbcalibration_nanoseconds_rbeventmemoryview,
            "Apply timing calibration to adc values of a specific channel",
            py::arg("event"), py::arg("channel"))
       .def("voltages",         wrap_rbcalibration_voltages_rbevent,
            "Apply voltage calibration to adc values of a specific channel",
            py::arg("event"), py::arg("channel"))
       .def("voltages",         wrap_rbcalibration_voltages_rbeventmemoryview,
            "Apply voltage calibration to adc values of a specific channel",
            py::arg("event"), py::arg("channel"))
       .def("from_bytestream",  &RBCalibration::from_bytestream, 
            "Deserialize a RBCalibration object from a Vec<u8>")
       .def("from_txtfile" ,    &RBCalibration::from_txtfile,
            "Initialize the RBCalibration from an ASCII file with the calibration constants (one per bin per channel)")
       .def("__repr__",        [](const RBCalibration &cali) {
                                  return "<RBCalibration : board id "
                                  + std::to_string(cali.rb_id) + ">";
                                  }) 
   ;

   // I/O functions
   m.def("get_tofpackets", &wrap_get_tofpackets_from_stream, "Get TofPackets from list of bytes");
   m.def("get_tofpackets", &wrap_get_tofpackets_from_file, "Get TofPackets from a file on disk");
   m.def("get_rbeventmemoryviews",       &wrap_get_rbeventmemoryviews_from_stream,
                                          "Get RBEventMemoryViews from list of bytes.\nArgs:\n * bytestream [list of char]\n * pos - start at position in stream\n * omit_duplicates [optional, bool] - reduced duplicate events (costs performance)",
                                          py::arg("bytestream"), py::arg("pos"), py::arg("omit_duplicates") = false);
   m.def("get_rbeventmemoryviews",        &wrap_get_rbeventmemoryviews_from_file,
                                          "Get RBEventMemoryViews from a file on disk.\nArgs:\n * filename - full path to file on disk as written by the readoutboards/liftof.\n              This file can only contain RBEventMemoryViews ('Blobs') without any other wrapper packets.\n * omit_duplicates - set this flag if you want to eeliminate duplicate events in the file. (Costs performance).",
                                          py::arg("filename"), py::arg("omit_duplicates") = false);
   m.def("unpack_tofevents",              &wrap_unpack_tofevents_from_tofpackets_from_stream,
                                          "Get TofEvents directly from list of bytes but the bytes are encoded TofPackets..\nArgs:\n * bytestream [list of char]\n * pos - start at position in stream\n",
                                          py::arg("bytestream"), py::arg("pos"));
   m.def("unpack_tofevents",              &wrap_unpack_tofevents_from_tofpackets_from_file,
                                          "Get TofEvents from a file on disk containing TofPackets. In case the packets contain RBEvents, they will be unpacked automatically.\nArgs:\n * filename - full path to file on disk containing serialized TofPackets.",
                                          py::arg("filename"));
   m.def("get_event_ids_from_raw_stream", &get_event_ids_from_raw_stream);
   m.def("get_bytestream_from_file",      &get_bytestream_from_file);
   m.def("get_nevents_from_file",         &get_nevents_from_file);
   m.def("ReadEvent",                     &read_event_helper);
   m.def("get_rbeventheaders",            &get_rbeventheaders); 
   
   //m.def("encode_blobevent",      &blobevent_encoder);
   //m.def("decode_blobevent",      &blobevent_decoder);   
   m.def("get_current_blobevent_size", &get_current_blobevent_size);

   // functions to read and parse blob files
   m.def("splice_readoutboard_datafile",   &splice_readoutboard_datafile);

   // Calibration functions
   m.def("remove_spikes",            &remove_spikes_helper);
   m.def("read_calibration_file",    &read_calibration_file);
   m.def("get_offsets",              &offset_getter);
   m.def("get_vincs",                &increment_getter);
   m.def("get_vdips",                &dip_getter);
   m.def("get_tbins",                &tbin_getter);
   // waveform stuff
   m.def("calculate_pedestal",       &calculate_pedestal_helper);
}
