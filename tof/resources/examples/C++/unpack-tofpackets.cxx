/**
 * Binary to unpack tofpackets/raw rb data to illustrate 
 * how to work with teh API
 * 
 * September 2023, gaps-online-sw V0.7
 * The API will not be stable until V1.0 and is thus 
 * subject to change. Please refer to the respective 
 * README.md
 *
 */

#include <iostream>
#include "cxxopts.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "io.hpp"
#include "calibration.h"

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("unpack-tofpackets", "Unpack example for .tof.gaps files with TofPackets.");
  options.add_options()
  ("h,help", "Print help")
  ("c,calibration", "Calibration file (in txt format)", cxxopts::value<std::string>()->default_value(""))
  ("file", "A file with TofPackets in it", cxxopts::value<std::string>())
  ("v,verbose", "Verbose output", cxxopts::value<bool>()->default_value("false"))
  ;
  options.parse_positional({"file"});
  auto result = options.parse(argc, argv);
  if (result.count("help")) {
    std::cout << options.help() << std::endl;
    exit(EXIT_SUCCESS);
  }
  if (!result.count("file")) {
    spdlog::error("No input file given!");
    std::cout << options.help() << std::endl;
    exit(EXIT_FAILURE);
  }
  auto fname   = result["file"].as<std::string>();
  bool verbose = result["verbose"].as<bool>();
  // -> Gaps relevant code starts here
 
  auto calname = result["calibration"].as<std::string>();
  RBCalibration cali;
  if (calname != "") {
    // obviously here we have to get all the calibration files, 
    // but for the sake of the example let's use only one
    // Ultimatly, they will be stored in the stream.
    spdlog::info("Will use calibration file {}", calname);
    cali = RBCalibration::from_txtfile(calname);
  }

  // the reader is something for the future, when the 
  // files get bigger so they might not fit into memory
  // at the same time
  //auto reader = Gaps::TofPacketReader(fname); 
  // for now, we have to load the whole file in memory
  auto packets = get_tofpackets(fname);
  spdlog::info("We loaded {} packets from {}", packets.size(), fname);

  u32 n_rbcalib = 0;
  u32 n_rbmoni  = 0;
  u32 n_mte     = 0;
  u32 n_tcmoni  = 0;
  u32 n_mtbmoni = 0;
  u32 n_unknown = 0;
  u32 n_tofevents = 0;
  for (auto const &p : packets) {
    // print it
    std::cout << p << std::endl;
    // there will be a more generic way to unpack TofPackets in the future
    // for now we have to use the packet_type field
    switch (p.packet_type) {
      case PacketType::RBCalibration : {
        // if you have the packet payload, the second argument 
        // (position in stream) will always be 0
        //
        // pos keeps track of the current position in bytestream, 
        // thus passed by reference so we need an rvalue
        //
        // the usize is a typedef from tof_typedefs.h and used
        // to make the rust and C++ code look more similar, so that 
        // is easier to compare them.
        usize pos = 0;
        auto cali = RBCalibration::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << cali << std::endl;
        }
        n_rbcalib++;
        break;
      }
      // this only works for the data I combined
      // recently, NOT for the "stream" kind of data
      // THe format will change as well soon.
      case PacketType::TofEvent : {
        usize pos = 0;
        auto ev = TofEvent::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << ev << std::endl;
          for (auto const &rbid : ev.get_rbids()) {
            RBEvent rb_event = ev.get_rbevent(rbid);
            if ((calname != "") && cali.rb_id == rbid ){
              // Vec<f32> is a typedef for std::vector<float32>
              Vec<Vec<f32>> volts = cali.voltages(rb_event, true); // second argument is for spike cleaning
                                                              // (C++ implementation still causes a 
                                                              // segfault sometimes
              Vec<Vec<f32>> times = cali.nanoseconds(rb_event);
              // volts and times are now ch 0-8 with the waveforms
              // for this event.

            }
            std::cout << rb_event << std::endl;
          }
        }
        n_tofevents++;
        break;
      }
      case PacketType::RBMoni : {
        usize pos = 0;
        auto moni = RBMoniData::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << moni << std::endl;
        }
        n_rbmoni++;
        break;
      }
      case PacketType::MasterTrigger : {
        usize pos = 0;
        auto mte = MasterTriggerEvent::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << mte << std::endl;
        }
        n_mte++;
        break;
      }
	/*case PacketType::TOFCmpMoni : {
        usize pos = 0;
        auto tcmoni = TofCmpMoniData::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << tcmoni << std::endl;
        }
        n_tcmoni++;
        break;
	}*/
      case PacketType::MTBMoni : {
        usize pos = 0;
        auto mtbmoni = MtbMoniData::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << mtbmoni << std::endl;
        }
        n_mtbmoni++;
        break;
      }
      default : {
        if (verbose) {
          std::cout << "-- nothing to do for " << p.packet_type << " --" << std::endl;
        }
        n_unknown++;
        break;
      }
    }
  }
  
  std::cout << "-- -- packets summary:" << std::endl;
  
  std::cout << "-- -- RBCalibration     : " << n_rbcalib << "\t (packets) " <<  std::endl;
  std::cout << "-- -- RBMoniData        : " << n_rbmoni  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- MasterTriggerEvent: " << n_mte     << "\t (packets) " <<  std::endl;
  std::cout << "-- -- TofEvent          : " << n_tofevents  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- TofCmpMoniData    : " << n_tcmoni  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- MtbMoniData       : " << n_mtbmoni << "\t (packets) " <<  std::endl;
  std::cout << "-- -- undecoded         : " << n_unknown << "\t (packets) " <<  std::endl;

  spdlog::info("Finished");
  return EXIT_SUCCESS;
}
