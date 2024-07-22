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

#include "legacy.h"
//#include "analysis.h"
#include <vector>

const int NRB   = 50; // Technically, it is 49, but we don't use 0
const int NCH   = 9;
const int NTOT  = (NCH) * NRB; // NTOT is the number of channels
const int NPADS = NTOT/2;        // NPAD: 1 per 2 SiPMs

double FitSine(std::vector<double> volts, std::vector<double> times)
//if you want to get all three fit parameters:
//std::vector<double> FitSine(std::vector<double> volts, std::vector<double> times, float cm)
{
  //float ns_off = 0; //cm*0.08; //Harting cable signal propagation is supposed to be 5.13 ns/m or 0.0513 ns/cm. crude measurement gives  0.08 ns/cm
  int start_bin = 20;
  int size_bin = 900; //can probably make this smaller

  int data_size = 0;
  double pi = 3.14159265;
  double a;
  double b;
  //if you want to get all fit params
  //double c;
  double p[3]; // product of fitting equation
  double XiYi = 0.0;
  double XiZi = 0.0;
  double YiZi = 0.0;
  double XiXi = 0.0;
  double YiYi = 0.0;
  double Xi = 0.0;
  double Yi = 0.0;
  double Zi = 0.0;
  double xi = 0.0;
  double yi = 0.0;
  double zi = 0.0;

  for(int i=start_bin; i < start_bin+size_bin; i++)
  {

// condition left over from when the sine wave was truncated 
//    if (volts[i] > -80.0)
//    {

      xi = cos(2*pi*0.02*(times[i]));  //for this fit we know the frequency is 0.02 waves/ns
      yi = sin(2*pi*0.02*(times[i]));
      zi = volts[i];
      XiYi += xi*yi;
      XiZi += xi*zi;
      YiZi += yi*zi;
      XiXi += xi*xi;
      YiYi += yi*yi;
      Xi   += xi;
      Yi   += yi;
      Zi   += zi;
      data_size++;
//    }
  }

  double A[3][3];
  double B[3][3];
  double X[3][3];
  double x = 0;
  double n = 0; //n is the determinant of A

  //the matrix A is XTX where X is the matrix of dimensions (data_size x 3) <cos(2pifreq*time), sin(2pifreq*time),1>
  A[0][0] = XiXi;
  A[0][1] = XiYi;
  A[0][2] = Xi;
  A[1][0] = XiYi;
  A[1][1] = YiYi;
  A[1][2] = Yi;
  A[2][0] = Xi;
  A[2][1] = Yi;
  A[2][2] = data_size;

  n += A[0][0] * A[1][1] * A[2][2];
  n += A[0][1] * A[1][2] * A[2][0];
  n += A[0][2] * A[1][0] * A[2][1];
  n -= A[0][0] * A[1][2] * A[2][1];
  n -= A[0][1] * A[1][0] * A[2][2];
  n -= A[0][2] * A[1][1] * A[2][0];
  x = 1.0/n;

  //find cofactor matrix of A, call this B
  B[0][0] =  (A[1][1] * A[2][2]) - (A[2][1] * A[1][2]);
  B[0][1] = ((A[1][0] * A[2][2]) - (A[2][0] * A[1][2])) * (-1);
  B[0][2] =  (A[1][0] * A[2][1]) - (A[2][0] * A[1][1]);
  B[1][0] = ((A[0][1] * A[2][2]) - (A[2][1] * A[0][2])) * (-1);
  B[1][1] =  (A[0][0] * A[2][2]) - (A[2][0] * A[0][2]);
  B[1][2] = ((A[0][0] * A[2][1]) - (A[2][0] * A[0][1])) * (-1);
  B[2][0] =  (A[0][1] * A[1][2]) - (A[1][1] * A[0][2]);
  B[2][1] = ((A[0][0] * A[1][2]) - (A[1][0] * A[0][2])) * (-1);
  B[2][2] =  (A[0][0] * A[1][1]) - (A[1][0] * A[0][1]);

  //take the transpose of the cofactor matrix and divide by the determinant to get the inverse matrix X
  for(int i=0;i<3;i++)
  {
    for(int j=0;j<3;j++)
    {
      X[i][j] = B[j][i] * x;
    }
  }

  //multiply p = zTX by the result
  p[0] = XiZi;
  p[1] = YiZi;
  p[2] = Zi;
  a = X[0][0] * p[0] + X[1][0] * p[1] + X[2][0] * p[2];
  b = X[0][1] * p[0] + X[1][1] * p[1] + X[2][1] * p[2];
  //offset parameter
  //c = X[0][2] * p[0] + X[1][2] * p[1] + X[2][2] * p[2];
  
  double phi = atan2(a,b);
  
  return phi;

  //amplitude parameter
  //double amp2 = pow(a,2)+pow(b,2);
  
  //return all three params
  //std::vector<double> v;
  //v.push_back(phi);
  //v.push_back(amp2);
  //v.push_back(c);

  //return v;
}

///mnt/tof-nas/nevis-data/tofdata/calibration/latest/
int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("unpack-tofpackets", "Unpack example for .tof.gaps files with TofPackets.");
  options.add_options()
  ("h,help", "Print help")
  ("c,calibration", "Calibration file (in txt format)", cxxopts::value<std::string>()->default_value("/mnt/tof-data/ucla-test-stand/ucla-test-stand-MAY/calib/"))
  ("file", "A file with TofPackets in it", cxxopts::value<std::string>())
  ("f,files", "List of Files", cxxopts::value<bool>()->default_value("false"))
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
  bool files   = result["files"].as<bool>();
  bool verbose = result["verbose"].as<bool>();

  FILE *fp;
  char tmpline[500];
  std::string fnames[1000];
  int j=0;
  if (files) {
    fp = fopen(fname.c_str(), "r");
    if (fp != NULL) {
      while (fscanf(fp, "%s", tmpline) != EOF) fnames[j++] = tmpline;
      fclose(fp);
    } else {
      printf("Unable to open file %s\n", fname.c_str());
    }
  } else {
    fnames[j++] = fname;
  }

  // -> Gaps relevant code starts here
 
  auto calname = result["calibration"].as<std::string>();
  RBCalibration cali[NRB]; // "cali" stores values for one RB

  // To read calibration data from individual binary files, when -c is
  // given with the directory of the calibration files
  if (calname != "") {
    for (int i=1; i<NRB; i++) {
      // First, determine the proper RB filename from its number
      std::string f_str;
      if (i<10) // Little Kludgy, but it works
	f_str = calname + "rb_0" + std::to_string(i) + ".cali.tof.gaps";
      else
	f_str = calname + "rb_" + std::to_string(i) + ".cali.tof.gaps";
      //spdlog::info("Extracting RB data from file {}", f_str);
      
      // Read the packets from the file
      //if ( std::filesystem::exists(f_str) ) {
      //printf("%s file exists\n", f_str.c_str() );
      //}
      // Before proceeding, check that the file exists. 
      struct stat buffer; 
      if ( stat(f_str.c_str(), &buffer) != -1 ) {
	auto packet = get_tofpackets(f_str);
	spdlog::info("We loaded {} packets from {}", packet.size(), f_str);
	// Loop over the packets (should only be 1) and read into storage
	for (auto const &p : packet) {
	  if (p.packet_type == PacketType::RBCalibration) {
	    // Should have the one calibration tofpacket stored in "packet".
	    usize pos = 0;
	    cali[i] = RBCalibration::from_bytestream(p.payload, pos); 
	  }
	}
      } //else {printf("File does not exist: %s\n", f_str.c_str());}
    }
  }
  
  // To read calibration data from individual text files, when -c is
  // given with the directory of the calibration files
  /*if (calname != "") {
    // obviously here we have to get all the calibration files, 
    // but for the sake of the example let's use only one
    // Ultimatly, they will be stored in the stream.
    for (int i=1; i<NRB; i++) {
      std::string f_str;
      if (i<10) // Little Kludgy, but it works
	f_str = calname + "/txt-files/rb0" + std::to_string(i) + "_cal.txt";
      else
	f_str = calname + "/txt-files/rb" + std::to_string(i) + "_cal.txt";
      
      //spdlog::info("Will use calibration file {}", calname);
      //cali[i] = RBCalibration::from_txtfile(calname);
      spdlog::info("Will use calibration file {}", f_str);
      cali[i] = RBCalibration::from_txtfile(f_str);
    }
    }*/

  // the reader is something for the future, when the 
  // files get bigger so they might not fit into memory
  // at the same time
  //auto reader = Gaps::TofPacketReader(fname); 
  // for now, we have to load the whole file in memory
  //auto packets = get_tofpackets(fname);
  //spdlog::info("We loaded {} packets from {}", packets.size(), fname);

  u32 n_rbcalib = 0;
  u32 n_rbmoni  = 0;
  u32 n_mte     = 0;
  u32 n_tcmoni  = 0;
  u32 n_mtbmoni = 0;
  u32 n_unknown = 0;
  u32 n_tofevents = 0;
  u32 highrms = 0;

  for (int k=0; k<j; k++) {
    auto packets = get_tofpackets(fnames[k]);
    spdlog::info("We loaded {} packets from {}", packets.size(), fnames[k]);

  for (auto const &p : packets) {
    // print it
    //std::cout << p.packet_type << std::endl;
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
	// We need a structure to hold the waveforms for an event. We
	// initialize and delete them with each new event
	GAPS::Waveform *wave[NTOT];
	GAPS::Waveform *wch9[NRB];
	float Ped_low   = 10;
	float Ped_win   = 70;
	float CThresh   = 10.0;
	float CFDS_frac = 0.25;
	float Qwin_low  = 75;
	float Qwin_size = 200;
	float Ped[NTOT];
	float PedRMS[NTOT];
	float Qint[NTOT];
	float VPeak[NTOT];
	float TCFDS[NTOT];
	bool  IsHit[NTOT] = {false};
	float phi[NRB];
	//use if you want all 3 fitting parameters
	//float amp[NRB];
	//float offs[NRB];
	float H_len[NRB];
	float shift[NRB];

	//in flight we should probably have array H_len[NRB] and read from database, for now i am manually setting the relavent channels
	//H_len[47] = 300; //Harting cable length in cm at UCLA
	//H_len[48] = 500;
	//H_len[37] = 305;
	
        auto ev = TofEvent::from_bytestream(p.payload, pos);
	unsigned long int evt_ctr = ev.mt_event.event_id;
	//printf("Event %ld: RBs -", evt_ctr);
	for (auto const &rbid : ev.get_rbids()) {
	  RBEvent rb_event = ev.get_rbevent(rbid);
	  // Now that we know the RBID, we can set the starting ch_no
	  // Eventually we will use a function to map RB_ch to GAPS_ch
	  usize ch_start = (rbid-1)*NCH; // first RB is #1
	  //usize rb_index = rbid-1;       // seems like RB1 should be at position 0, etc...
	  if (verbose) {
	    std::cout << rb_event << std::endl;
          }
	  Vec<Vec<f32>> volts;
	  Vec<Vec<f32>> times;
	  //if ((calname != "") && cali.rb_id == rbid ){
	  if (calname != "") { // For combined data all boards calibrated
	    // Vec<f32> is a typedef for std::vector<float32>
	    volts = cali[rbid].voltages(rb_event, false); //second argument is for spike cleaning
	    // (C++ implementation causes a segfault sometimes when "true"
	    times = cali[rbid].nanoseconds(rb_event);
	    // volts and times are now ch 0-8 with the waveform for this event.

	    // First, store the waveform for channel 9
	    Vec<f64> ch9_volts(volts[8].begin(), volts[8].end());
	    Vec<f64> ch9_times(times[8].begin(), times[8].end());
	    wch9[rbid] = new GAPS::Waveform(ch9_volts.data(),ch9_times.data(),rbid,0);
	    wch9[rbid]->SetPedBegin(Ped_low);
	    wch9[rbid]->SetPedRange(Ped_win);
	    wch9[rbid]->CalcPedestalRange(); 
	    float ch9RMS = wch9[rbid]->GetPedsigma();
	    //printf(" %d(%.1f)", rbid, ch9RMS);
	    // printf(" %d", rbid);
	      
	    // Now, deal with all the SiPM data
	    for(int c=0;c<NCH-1;c++) {
	      usize cw = c+ch_start; 

	      Vec<f64> ch_volts(volts[c].begin(), volts[c].end());
	      Vec<f64> ch_times(times[c].begin(), times[c].end());
	      wave[cw] = new GAPS::Waveform(ch_volts.data(),ch_times.data() ,cw,0);
	      
	      // Calculate the pedestal
	      wave[cw]->SetPedBegin(Ped_low);
	      wave[cw]->SetPedRange(Ped_win);
	      wave[cw]->CalcPedestalRange(); 
	      wave[cw]->SubtractPedestal(); 
	      Ped[cw] = wave[cw]->GetPedestal();
	      PedRMS[cw] = wave[cw]->GetPedsigma();

	      //if ( c==0 && (PedRMS[cw] > 15) && (ch9RMS < 190) ) {
		// RMS_ch1 has ch9 data && RMS_ch9 has normal data        
		//printf(" %ld Row %d: %8.1f %8.1f\n", evt_ctr, rbid, ch9RMS, PedRMS[cw]);
		//for(int j=0;j<8;j++) printf(" %8.1f",PedRMS[ch_start+j]);
		//printf("\n");
	      //}
	      //std::cout << "One" << std::endl;	      
	      //pedRMS cut at 1.0, probably only needs to be at 2.0 but doesn't make much difference
	      if (PedRMS[cw] > 1.0) {
		//documenting hi PedRMS in std::out and txt file
		highrms++;
		std::ofstream fileP;
                fileP.open ("HiRMS_feb_pb_RBS.csv", std::ios::app);
                fileP << evt_ctr << "," << cw << std::endl;
                fileP.close();

		continue;
	      }
	      //std::cout << "Two" << std::endl;
	      // Set thresholds and find pulses
	      wave[cw]->SetThreshold(CThresh);
	      wave[cw]->SetCFDSFraction(CFDS_frac);
	      VPeak[cw] = wave[cw]->GetPeakValue(Qwin_low, Qwin_size);
	      Qint[cw]  = wave[cw]->Integrate(Qwin_low, Qwin_size);
	      wave[cw]->FindPeaks(Qwin_low, Qwin_size);
	      //if ( (wave[cw]->GetNumPeaks() > 0) && (Qint[cw] > 5.0) ) {
	      if ( (wave[cw]->GetNumPeaks() > 0) ) {
		//printf("%i\n",cw);
		IsHit[cw] = true;
		wave[cw]->FindTdc(0, GAPS::CFD_SIMPLE);       // Simple CFD
		TCFDS[cw] = wave[cw]->GetTdcs(0);
		//printf("%ld hit\n",cw);

		//phi[rbid] = FitSine(ch9_volts,ch9_times);
		
		phi[rbid] = FitSine(ch9_volts,ch9_times);
		
		// for all three fit params
		//std::vector<double> v = FitSine(ch9_volts,ch9_times,H_len);
                //phi[rbid] = v[0];
		//amp[rbid] = v[1];
		//offs[rbid] = v[2]; 

		//printf("EVT %12ld - ch %3ld: %10.5f\n", evt_ctr, cw, TCFDS[cw]);
	      }	//end "if channel is hit" loop	
	    }  //end channel loop (8)

	    //inside this loop, need to define first board as board A and compare all other phase shifts to board A
	    //THIS IS NOT WORKING CODE, THIS IS AN OUTLINE OF WHAT THE CODE SHOULD DO! 
	    //I was doing this in python and with only 2 boards before
	    
	    /* if (firstRB)
	     * {
	     *   float phiA = phi[rbid];
	     * } 
	     * float phi_shift=phiA-phi[rbid];                     //units of rad
	     * if(phi_shift < -pi/3){
             *   float shiftRB = (phi_shift+2*pi)/(2*pi*0.02); //ns
             * }
	     * else if(phi_shift-H_shift > pi/3){
             *   float shiftRB = (phi_shift-2*pi)/(2*pi*0.02); //ns
             * }
	     * else{
             *   float shiftRB = (phi_shift)/(2*pi*0.02);      //ns
             * }
	     *
	     * shift[rbid] = shiftRB;
	     */ 
	  }   //end rb loop
	}
	//printf("\n");
	// Now that we have all the waveforms in place, we can analyze
	// the event. Start by looping over all paddles, and process
	// any paddles with hits
	for (int k=0; k<NPADS; k++) {
	  //First, check with the MTB data to see if paddle is hit

	  // Then find the pulse information from each SiPM
	  int ch0 = k*2, ch1 = k*2+1;
	  /*
	  Paddle[k].time_a = wave[ch0].FindTdc(1,CFD_SIMPLE);
	  Paddle[k].time_b = wave[ch1].FindTdc(1,CFD_SIMPLE);
	  Paddle[k].peak_a = wave[ch0].GetPeakValue(100, 200);
	  Paddle[k].peak_b = wave[ch1].GetPeakValue(100, 200);
	  Paddle[k].charge_a = wave[ch0].Integrate(100, 200);
	  Paddle[k].charge_b = wave[ch1].Integrate(100, 200);
	  // Also hit position, min_i, and t_avg
	  */
	}
	// Now calculate beta, charge, and inner/outer tof x,y,z


	//SNF jan 2023, calculate timing for UCLA test stand setup
	//run 16 /mnt/tof-nas/ucla-test-stand-JAN/16/ configuration:
	//trigger = 'or' of U1A and U1B
	//U1A signal split and sent to RB47 ch0 and RB48 ch0
	//U1B signal split and sent to RB37 ch0 and RB37 ch1
	
	// start with just looking at U1B signal: ch0 is 324 in wave vector, ch1 is 325
        
        //std::cout << "Three" << std::endl;
	//std::cout << "HIT 432, 433, 434, 435, 436, 437: "<<"," << IsHit[432] << "," << IsHit[433] << "," << IsHit[434] << "," << IsHit[435]  <<"," << IsHit[436] << "," << IsHit[437] << "," << std::endl;
	if (IsHit[432] && IsHit[433] && IsHit[434] && IsHit[435]) {
	
	/*  if (TCFDS[324] < 90.0 || TCFDS[325] < 90.0) {
	    std::ofstream file2;
            file2.open ("RB37_tdc0_feb.csv", std::ios::app);
            file2 << evt_ctr;
            file2 << "," << TCFDS[324] << "," << TCFDS[325] << std::endl;
            file2.close();
          }
	  else { */
            std::ofstream myfile;
            myfile.open ("sig131.csv", std::ios::app);
            myfile << evt_ctr;
            //myfile << "," << TCFDS[432] << "," << TCFDS[433] << "," << phi[49] << "," << TCFDS[326] << "," << TCFDS[327] << "," << phi[37] << std::endl;
	    myfile << "," << TCFDS[432] << "," << TCFDS[433] << "," << TCFDS[434] << "," << TCFDS[435] << std::endl;
            myfile.close();
	    //std::cout << "Four" << std::endl;
	  //}
		  
	}
        
						
        // now look at U1A signal: RB47 ch0 is 414 in wave vector, RB48 ch0 is 423
  
        if (IsHit[414] && IsHit[423]) {

       /* if (TCFDS[414] < 90.0 || TCFDS[423] < 90.0) {
            std::ofstream file3;
            file3.open ("RB4748_tdc0_feb.csv", std::ios::app);
            file3 << evt_ctr;
            file3 << "," << TCFDS[414] << "," << TCFDS[423] << std::endl;
            file3.close();
          }

          else {
*/
            std::ofstream myfile4;
            myfile4.open ("RB4748_pb_RBS.csv", std::ios::app);
            myfile4 << evt_ctr;
            //myfile4 << "," << TCFDS[414] << "," << TCFDS[423] << "," << phi[47] << "," << phi[48] << "," << amp[47] << "," << amp[48] << "," << offs[47] << "," << offs[48] << std::endl;
            myfile4 << "," << TCFDS[414] << "," << TCFDS[423] << "," << phi[47] << "," << phi[48] << std::endl;
	    myfile4.close();
  //        }
                  
        }

	
/*
	std::ofstream myfile;
        myfile.open ("PEDs.csv", std::ios::app);
	myfile << evt_ctr;
	for (int i; i++; i<NTOT){
          myfile << Ped[i] << ",";
        }
	myfile << std::endl;
	myfile.close();

	std::ofstream myfile2;
        myfile2.open ("PEDRMS.csv", std::ios::app);
        myfile2 << evt_ctr;
        for (int i; i++; i<NTOT){
          myfile2 << PedRMS[i] << ",";
        }
        myfile2 << std::endl;
	myfile2.close();

	std::ofstream myfile3;
        myfile3.open ("Vpeaks.csv", std::ios::app);
        myfile3 << evt_ctr;
        for (int i; i++; i<NTOT){
          myfile3 << VPeak[i] << ",";
        }
        myfile3 << std::endl;
	myfile3.close();
*/
	n_tofevents++;
	//printf("%i\n",n_tofevents);
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
      case PacketType::CPUMoniData : {
        usize pos = 0;
        auto tcmoni = CPUMoniData::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << tcmoni << std::endl;
	}
        n_tcmoni++;
	break;
      }
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
  } 
  std::cout << "-- -- packets summary:" << std::endl;
  
  std::cout << "-- -- RBCalibration     : " << n_rbcalib << "\t (packets) " <<  std::endl;
  std::cout << "-- -- RBMoniData        : " << n_rbmoni  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- MasterTriggerEvent: " << n_mte     << "\t (packets) " <<  std::endl;
  std::cout << "-- -- TofEvent          : " << n_tofevents  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- TofCmpMoniData    : " << n_tcmoni  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- MtbMoniData       : " << n_mtbmoni << "\t (packets) " <<  std::endl;
  std::cout << "-- -- undecoded         : " << n_unknown << "\t (packets) " <<  std::endl;
  std::cout << "-- -- High RMS         : " << highrms << "\t (RB events) " <<  std::endl;

  spdlog::info("Finished");
  return EXIT_SUCCESS;
}
