/**
 * Binary to illustrate how to use the tof paddle 
 * database.
 * To use this example, the code has to be build with
 * BUILD_CXXDB=ON
 * 
 * March 2025, gaps-online-sw V0.10
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
#include "database.h"

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("show-paddles", "List all paddles as they are saved in te database");
  options.add_options()
  ("h,help", "Print help")
  ;
  auto result = options.parse(argc, argv);
  if (result.count("help")) {
    std::cout << options.help() << std::endl;
    exit(EXIT_SUCCESS);
  }
  
  auto paddles = Gaps::get_tofpaddles();
  for (auto const &p : paddles) {
    std::cout << "************* PADDLE " << (int)p.first << "***********" << std::endl;
    std::cout << p.second << std::endl;
    std::cout << "\n\n" << std::endl;
  }
  spdlog::info("Finished");
  return EXIT_SUCCESS;
}
