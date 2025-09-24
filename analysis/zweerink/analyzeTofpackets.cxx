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
#include <vector>

#include "include/constants.h"

double FitSine(std::vector<double> volts, std::vector<double> times);

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("unpack-tofpackets", "Unpack example for .tof.gaps files with TofPackets.");
  options.add_options()
  ("h,help", "Print help")
  ("c,calibration", "Calibration file (in txt format)", cxxopts::value<std::string>()->default_value("/home/gaps/csbf-data/calib/latest/"))
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
  RBCalibration cali[NRB]; // "cali" stores values for one RB

  // To read calibration data from individual binary files, when -c is
  // given with the directory of the calibration files. Since the
  // calibration files for each RB change with each calibration run,
  // this code reads the list of calibration files in the directory,
  // determines the RB number and copies the string into the relevant
  // array position. For RBs with no calibration file, the length of
  // the entry will be 0. We then read the calibrations for all RBs
  // with files.
  bool RB_Calibrated[NRB] = { false };
  std::string cnames[NRB];
  if (calname != "") {
    char pname[500], line[500];
    snprintf(pname,450, "ls %s/RB*.cali.tof.gaps", calname.c_str());
    FILE *fp = popen(pname, "r");
    while (fscanf(fp,"%s", line) != EOF) {
      std::string c_name(line);          // Calib file found
      int position = c_name.find("RB");  // Find "RB" in the name
      std::string rbstr = c_name.substr(position+2, 2); // Extract RB num
      int rbnum = atoi(rbstr.data());    // Convert to integer
      //printf("%s %d %s\n", rbstr.c_str(), rbnum, line);
      cnames[rbnum] = c_name;            // Copy to proper place in array
    }
    pclose(fp);
    // Print out the calibration filenames as a sanity check
    //for (int i=0; i<NRB; i++) {
    //printf("%d: %lu %s\n", i, cnames[i].size(), cnames[i].c_str());
    //}

    for (int i=1; i<NRB; i++) {
      if (cnames[i].size() > 4) { // RB has a calibration file
	std::string f_str = cnames[i];
	
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
	      RB_Calibrated[i] = true;
	    }
	  }
	}
      }
    }
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
    //std::cout << p << std::endl;
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
	
#define OLD_CHECK 0
#if OLD_CHECK
	GAPS::Waveform *wave[NTOT];
	GAPS::Waveform *wch9[NRB];
#endif
	float Ped_low   = 350;
	float Ped_win   = 100;
	float CThresh   = 10.0;
	float CFDS_frac = 0.25;
	float Qwin_low  =  10;
	float Qwin_size = 190;
	double Ped[NTOT];
	double PedRMS[NTOT];
	double Qint[NTOT];
	double VPeak[NTOT];
	double TDC[NTOT];
	double X_POS[NPAD];
	float  Phi[NRB];
	bool  IsHit[NTOT] = {false} ;
	int NPadCube = 0;
	int NPadUmb  = 0;
	int NPadCort = 0;
	
        auto ev = TofEvent::from_bytestream(p.payload, pos);
	unsigned long int evt_ctr = ev.mt_event.event_id;
	//printf("Event %ld: RBs -", evt_ctr);
	//printf("Event %ld\n", evt_ctr);
	for (auto const &rbid : ev.get_rbids()) {
	  RBEvent rb_event = ev.get_rbevent(rbid);
	  // Now that we know the RBID, we can set the starting ch_no
	  // Eventually we will use a function to map RB_ch to GAPS_ch
	  usize ch_start = (rbid-1)*NCH; // first RB is #1
          // Let's also store the channel mask to use later.                   
          int ch_mask = rb_event.header.channel_mask;
	  if (verbose) {
	    std::cout << rb_event << std::endl;
	  }
	  Vec<Vec<f32>> volts;
	  Vec<Vec<f32>> times;
	  if (calname != "") { // For combined data all boards calibrated
	    // Vec<f32> is a typedef for std::vector<float32>
	    volts = cali[rbid].voltages(rb_event, false);
	    // second argument is for spike cleaning (C++
	    // implementation causes a segfault sometimes when "true"
	    times = cali[rbid].nanoseconds(rb_event);
	    // volts and times are now ch 0-8 with the waveform for this event.

	    // First, store the waveform for channel 9
	    Vec<f64> ch9_volts(volts[8].begin(), volts[8].end());
	    Vec<f64> ch9_times(times[8].begin(), times[8].end());
	    // Calculate the ch9 phase. For now, if we have ch9 data
            // for this RB, we want to analyze it.
	    // NEW STUFF FOR ACHIM
            Phi[rbid] = FitSine(ch9_volts,ch9_times);
	    // DONE WITH NEW STUFF
	    //printf(" %d", rbid); fflush(stdout);
	      
	    // Now, deal with all the SiPM data
	    for(int c=0;c<NCH;c++) {
	      usize cw = c+ch_start; 
	      unsigned int inEvent = ch_mask & (1 << c);
              if (inEvent > 0 ) {
		
		Vec<f64> ch_volts(volts[c].begin(), volts[c].end());
		Vec<f64> ch_times(times[c].begin(), times[c].end());
#if OLD_CHECK
		wave[cw] = new GAPS::Waveform(ch_volts.data(),ch_times.data() ,cw,0);
#endif
	    // NEW STUFF FOR ACHIM
		// Calculate the ped and pedrms for the channel
		int ctr=0, i=0;
		double sum=0.0, sum2=0.0; 
		while (ch_times[i++]<Ped_low); // Find start of ped window
		while (ch_times[i]<Ped_low+Ped_win) {
		  sum +=  ch_volts[i];
		  sum2 += ch_volts[i]*ch_volts[i];
		  ctr++;
		  i++;
		}
		double average;
		if (ctr>0) {
		  average = sum / (double)ctr;
		  PedRMS[cw] = sqrt (sum2/(double)ctr - average*average);
		  Ped[cw] = average;
		} else {
		  Ped[cw] = PedRMS[cw] = 0.0;
		}

		// Subtract Pedestal
		i=0;
		while (i<ch_times.size()-1 ) {// Move through trace
		  ch_volts[i++] -= Ped[cw];  // Subtract Pedestal
		}

		// Find VPeak 
		int    vpeak_bin=512;
		double vpeak_time=250.0, vpeak=0.0;
		// 512 and 250 are hardwired values to ensure the TDC
		// values outside of a window from 5-220ns (when no
		// pulses are in the trace) and don't interfere with
		// the pedestal window (350-450ns) either
		i=0;
		while (ch_times[i++]<Qwin_low); // Find start of pulse window
		while (ch_times[i]<Qwin_low+Qwin_size) {
		  sum=0.0;
		  for (int j=i-1; j<=i+1; j++) sum += ch_volts[j];
		  if (sum > vpeak) {
		    vpeak = sum;
		    if (ch_volts[i]>CThresh) {
		      vpeak_time = ch_times[i];
		      vpeak_bin = i;
		    }
		  }
		  i++;
		}
		VPeak[cw] = vpeak/3.0;  // Convert sum to average

		// Integrate charge in window from vpeak-20 to vpeak+80
		double charge=0.0;
		i=vpeak_bin;
		while (ch_times[i--] > vpeak_time-20.0); // Find start point
		while (ch_times[i]<vpeak_time+80) { // Integrate window
		  charge += ch_volts[i] * (ch_times[i]-ch_times[i-1]);
		  i++;
		}
		Qint[cw]  = charge / 50.0; // 50 Ohm impedance
		
		// Find TDC using simple CFD method.
		sum = 0.0;
		if (vpeak_bin>0) { // Only do if peak was above threshold
		  // Determine the threshold for finding the time. Use
		  // 25% of the average of VPeak and two adjacent bins
		  int idx = vpeak_bin;
		  for (int j=idx-1; j<=idx+1; j++) sum += ch_volts[j];
		  double tmp_thresh = CFDS_frac * (sum / 3.0);
		  
		  // Now scan through the waveform around the peak to
		  // find the bin crossing the calculated
		  // threshold. Bin idx (vpeak_time) is the peak so it
		  // is definitely above threshold. So let's walk
		  // backwards through the trace until we find a bin
		  // value less than the threshold.
		  int bin = ch_times.size();
		  for (i=idx; ch_times[i]>10; i--) {//Stay 10ns into trace
		    if ( ch_volts[i] < tmp_thresh ) {
		      bin = i;
		      i=0;
		    }
		  }
		  
		  // Finally, interpolate to find exact time where trace
		  // crossed the CFD threshold
		  double tdiff = ch_times[bin+1]-ch_times[bin];
		  double vdiff = ch_volts[bin+1]-ch_volts[bin];
		  double time = ch_times[bin] +
		    (tmp_thresh-ch_volts[bin])/vdiff * tdiff;
		  TDC[cw] = time;
		} else { // Trace never crosses CThresh
		  TDC[cw] = -1.0;
		}
		
		// TODO: Need to correct TDC values for measuring time
		// in buffer rather than time to exit buffer.
		// WILL GET THIS LATER; SHOULD BE A FEW LINES TO ADD
		
		// From the TDC values, calculate the x_pos along paddle
		if (cw%2 == 1) { // have analyzed both ends of paddle
		  double tdc_diff  = TDC[cw-1] - TDC[cw];
		  double x_pos = tdc_diff*154.0/2.0; // + -> toward cw
		  // ACHIM TODO: Determine paddle number/orientation for
		  // this cw,cw-1 pair.
		  //int pad = GET_PADDLE_ID;

		  // Once we know paddle/orientation, uncomment next 4 lines
		  /*
		  X_POS[pad] = orient*x_pos;
		  if(pad<61) NPadCube++;      // Paddle in Cube
		  else if (i<109) NPadUmb++;  // Paddle in Umbrella       
		  else if (i<161) NPadCort++; // Paddle in Cortina          
		  */
		}
	    // DONE WITH NEW STUFF
		printf("new %3ld: %7.3lf %7.3lf %7.3lf %7.3lf %7.3lf %7.3lf\n",
		       cw, Ped[cw], PedRMS[cw], VPeak[cw], Qint[cw],
		       TDC[cw], Phi[rbid]);

#if OLD_CHECK
		// Calculate the pedestal
		wave[cw]->SetPedBegin(Ped_low);
		wave[cw]->SetPedRange(Ped_win);
		wave[cw]->CalcPedestalRange(); 
		wave[cw]->SubtractPedestal(); 
		//Ped[cw] = wave[cw]->GetPedestal();
		//PedRMS[cw] = wave[cw]->GetPedsigma();
		
		
		// Set thresholds and find pulses
		wave[cw]->SetThreshold(CThresh);
		wave[cw]->SetCFDSFraction(CFDS_frac);
		//VPeak[cw] = wave[cw]->GetPeakValue(Qwin_low, Qwin_size);
		//Qint[cw]  = wave[cw]->Integrate(Qwin_low, Qwin_size);
		wave[cw]->FindPeaks(Qwin_low, Qwin_size);
		if ( (wave[cw]->GetNumPeaks() > 0) ) {
		  wave[cw]->FindTdc(0, GAPS::CFD_SIMPLE);       // Simple CFD
		  //TDC[cw] = wave[cw]->GetTdcs(0);
		}
		printf("old %3ld: %7.3lf %7.3lf %7.3lf %7.3lf %7.3lf\n",
		       cw, wave[cw]->GetPedestal(),
		       wave[cw]->GetPedsigma(),
		       wave[cw]->GetPeakValue(Qwin_low, Qwin_size),
		       wave[cw]->Integrate(Qwin_low, Qwin_size),
		       wave[cw]->GetTdcs(0) );
#endif		
	      }
	    }
	  }
	}
	//printf("\n");

	for (int k=0; k<NPAD; k++) {
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

double FitSine(std::vector<double> volts, std::vector<double> times)
//if you want to get all three fit parameters:                                 
//std::vector<double> FitSine(std::vector<double> volts, std::vector<double> times, float cm)                                                                 
{
  //float ns_off = 0; //cm*0.08; //Harting cable signal propagation is
  //supposed to be 5.13 ns/m or 0.0513 ns/cm. crude measurement gives
  //0.08 ns/cm
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

  for(int i=start_bin; i < start_bin+size_bin; i++) {
    xi = cos(2*pi*0.02*(times[i]));  //for this fit we know nu=0.02 waves/ns
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
  }

  double A[3][3];
  double B[3][3];
  double X[3][3];
  double x = 0;
  double n = 0; //n is the determinant of A                                    

  //the matrix A is XTX where X is the matrix of dimensions
  // (data_size x 3) <co\ s(2pifreq*time), sin(2pifreq*time),1>
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

  //take the transpose of the cofactor matrix and divide by the
  //determinant to get the inverse matrix X                                    
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
