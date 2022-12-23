#include <cassert>
#include <iostream>
#include <vector>
#include <climits>
#include <fstream>
#include <chrono>

#include "BlobFileServer.h"

#include "serialization.h"
#include "blobroutines.h"

#include "CraneLogging.hh"

// BlobEvt_t
#include "TOFCommon.h"

#include "zmq.hpp"

//#define NCHAN 8
TOF::BlobFileServer::BlobFileServer(std::vector<std::string> blobfilenames)
{
  blob_filenames_ = blobfilenames;
  std::cout << "[INFO] - got input blobfiles..." << std::endl;
  for (auto fname : blob_filenames_) std::cout << "-- " << fname << std::endl;
  // keep track of how many events we processed
  nevents_processed_ = 0;

  for (auto k : blob_filenames_)
    {
      std::cout << "[DEBUG] - Opening file " << k << std::endl;
      blob_files_.push_back(fopen(k.c_str(), "rb"));
    }
  nboards_ = blob_files_.size();
  for (auto k : blob_files_) 
    {
      if (k == nullptr)
        {
           std::cerr << "[FATAL] opening one of the blobfiles failed!" << std::endl;
           std::exit(1);
        }
    }
  // initailzie fields
  for (size_t k=0;k<MAX_BRDS;k++) status_[k] = 0;


  // files have to be closed and reopened
  for (int k=0;k<nboards_;k++) {
    FillFirstEvents(blob_files_[k], k, &times_);
    fclose(blob_files_[k]);
  }
  for (size_t k=0;k<blob_filenames_.size(); k++)
    {
      std::cout << "[DEBUG] - Re-pening file " << blob_filenames_[k] << std::endl;
      blob_files_[k] = fopen(blob_filenames_[k].c_str(), "rb");
    }
  times_.nbrds = nboards_;
  FindFirstEvents(&times_);
  int orig_time[nboards_];

  for (size_t k=0;k<nboards_;k++) orig_time[k] = 0;
  for (int k=0;k<nboards_;k++) {
    int e_ctr = 0;
    do { // Read to the first common event from each file
      status_[k] = ReadEvent(blob_files_[k], &event_[k]);
      //std::cout << "do-while " << event_[k].event_ctr << " " << e_ctr << " " << times_.first_evt[k] << std::endl;
      e_ctr++;
      nevents_processed_++;
    } while (e_ctr <= times_.first_evt[k]);
    //} while (e_ctr <= firstEvt[k]);
    orig_time[k] = event_[k].timestamp;          // Record the start time
    times_.first_evt_ID[k] = event_[k].event_ctr; // Record the evt_ID
  }
  //LoadCalibrations();
  board_in_event_mask_ = std::vector<int>(nboards_,0);
  std::cout << "[INFO] : initialization done!" << std::endl;

  // initialize variables for random event mode
  first_random_event_id_ = 10000000;  

  n_events_finish_criterion_  = 0;
  n_seconds_finish_criterion_ = 0;  
}

/*******************************************************/

TOF::BlobFileServer::~BlobFileServer()
{
  std::cout << "[INFO] Calling destructor, closing files.." << std::endl;
  for (auto &k : blob_files_)
    {
        if (k == nullptr) continue;    
        fclose(k);
    }
  std::cout << "[INFO] Destroying sockets" << std::endl;
  //for (auto k : sockets_)
  //  {zsock_destroy(&k);}
  std::cout << "[INFO] ..done" << std::endl;
}

/*******************************************************/

BlobEvt_t* TOF::BlobFileServer::GetNextEvent(){

  // remember, in the constructor we were already reading the
  // very first event, so we have one in the buffer already

  int inevent[nboards_];
  int curr_event[nboards_];
  for (size_t k=0;k<nboards_;k++) inevent[k]    = 0;
  for (size_t k=0;k<nboards_;k++) curr_event[k] = 0;
   
  std::cout << "[DEBUG] "<<blob_files_.size() << std::endl; 
  // copy the event in the buffer, so we can return it
  for (size_t k=0;k<times_.nbrds;k++)
    { event_buffer_[k] = event_[k];}
  for (unsigned int k=0; k<times_.nbrds ; k++)
    {
      curr_event[k] = event_[k].event_ctr - times_.first_evt_ID[k];
      //std::cout << "[DEBUG] evt ctr  " << times_.first_evt_ID[k] << std::endl;
      //std::cout << "[DEBUG] first ev " << event_[k].event_ctr << std::endl;
      //std::cout << "[DEBUG] curr evt " << curr_event[k] << std::endl;
    }
  BoardsInEvent(status_, curr_event, inevent, times_.nbrds);
  //std::cout << "---------------------------" << std::endl;
  std::vector<bool> all_files_ended(false, blob_files_.size());
  //bool all_files_ended = true;
  for (unsigned int k=0; k<blob_files_.size() ; k++)
    {
      std::cout << "[DEBUG] inevent " << inevent[k] << std::endl;
      
      // at the very last, read the next event if there is one more      
      //std::cout<<"[DEBUG] Reading events.."<< std::endl;
      if (inevent[k]==1 && status_[k] != -1 ) { // status=-1 -> EOF
          status_[k] = ReadEvent(blob_files_[k], &event_[k]);
          nevents_processed_++;
      }
      std::cout << status_[k] << std::endl;
      if (status_[k] == -1) all_files_ended[k] = true;
      std::cout<<"[DEBUG] .. done"<< std::endl;
    }  
  bool all_files_ended_sum = true;
  for (auto k : all_files_ended) all_files_ended_sum = all_files_ended_sum && all_files_ended[k];
  if (all_files_ended_sum) 
    {
      std::cout << "[INFO] All events read, returning nullptr" << std::endl;
      return nullptr;
    }

  uint32_t event_num;
  long min_event_id = INT_MAX;
  long max_event_id = -INT_MAX;
  bool success = EventIdSanityCheck(event_buffer_, inevent , event_num, min_event_id, max_event_id);
  if (!success)
    {std::cerr << "[ERROR] - different event ids " << min_event_id << " " << max_event_id << std::endl;}
  //std:: cout << "[DEBUG] - read events, reports status  < ";
  //for (auto s : status_)
  //  {std::cout << " " << s;}
  //std::cout << " >" << std::endl;
  //std:: cout << "[DEBUG] - boards participating in event  < ";
  //for (auto s : inevent)
  //  {std::cout << " " << s;}
  //std::cout << " >" << std::endl;
  for (int k=0;k<nboards_; k++) 
    {board_in_event_mask_[k] = inevent[k];}
  std:: cout << "[DEBUG] - returning events ";
  return event_buffer_;
}

/*************************************************/

std::vector<int> TOF::BlobFileServer::GetNextEventMask()
{
  return board_in_event_mask_;
}

/*************************************************/

bool TOF::BlobFileServer::EventIdSanityCheck(BlobEvt_t event[],
                                       int inevent[],
                                       unsigned int&  eventid,
                                       long& min_event,
                                       long& max_event)
{
  std::cout << "[DEBUG] Starting eventid sanity check" << std::endl;

  std::vector<long> allBoardEventId;
  //std::cout << "nboard "  << nbrds << std::endl;
  //std::cout << eventid << std::endl;
  //std::cout << min_event << std::endl;
  //std::cout << max_event << std::endl;
  for (int k=0; k<nboards_; k++) 
    {    
      //std::cout<< inevent[k] << std::endl;
      //std::cout << event[k].event_ctr << std::endl;
      //if (event[k].event_ctr == 0) continue; // 0 is typically for boards which are not present in the blob
      if (inevent[k] == 0) continue;// this board did not participate in the event id
      if (event[k].event_ctr < min_event) min_event = event[k].event_ctr;
      if (event[k].event_ctr > max_event) max_event = event[k].event_ctr;
      allBoardEventId.push_back(event[k].event_ctr);
    }    
    if ((allBoardEventId).size() == 0) 
        {}//std::cout << "vector empty" << std::endl;}
  if ( std::equal(allBoardEventId.begin() + 1, allBoardEventId.end(), allBoardEventId.begin()) )
     {    
      eventid = allBoardEventId[0];
      return true;
     }    
  else 
     {    
      std::cerr << "-----------------" << std::endl;
      for (auto k : allBoardEventId) { std::cerr << " -- " << k;}
      std::cerr << std::endl <<  "[ERROR] Event id not the same accross the boards!" << std::endl;
      eventid = 0; 
      return false;
     }    
}

/*******************************************/

void TOF::BlobFileServer::LoadCalibrations()
{

   // find relevant boards
  // TODO: change this to use event[k].id, eventually
  unsigned int       boardnums[MAX_BRDS]; // maps event board number to RB#
  unsigned long long boarddnas[MAX_BRDS] = { 
    77380906573213780, // 1
    9609908518406236,  // 2
    78985381379328092, // 3
    24942185850882132, // 4
    //25003068095826012, // 5
    //32831594097035348  // 6
  };  
  for (int k=0; k<times_.nbrds; k++)
    for (int i=0;i<MAX_BRDS;i++)
      if (event_[k].dna == boarddnas[i])
        boardnums[k] = i;

  for (int k=0; k<times_.nbrds; k++) {
    std::string calfilename = "../resources/calibrations/rb" + std::to_string(boardnums[k]+1) + "_cal.txt";
    std::fstream calfile(calfilename.c_str(), std::ios_base::in);
    if (calfile.fail()) {
      std::cerr << "[ERROR] Can't open " << calfilename << " - not calibrating" << std::endl;
      continue;
    }
    for (int i=0; i<NCHN; i++) {
      for (int j=0; j<NWORDS; j++)
        calfile >> calibrations_[k][i].vofs[j];
      for (int j=0; j<NWORDS; j++)
        calfile >> calibrations_[k][i].vdip[j];
      for (int j=0; j<NWORDS; j++)
        calfile >> calibrations_[k][i].vinc[j];
      for (int j=0; j<NWORDS; j++)
        calfile >> calibrations_[k][i].tbin[j];
    }
    board_has_cal_[k] = true;
  }

}

/***********************************************/

bool* TOF::BlobFileServer::BoardHasCalibration()
{
  return board_has_cal_;
}

/***********************************************/

std::vector<std::vector<Calibrations_t>> TOF::BlobFileServer::GetCalibrations()
{
  std::vector<std::vector<Calibrations_t>> cals(MAX_BRDS);
  long lSize;
  char * buffer;
  size_t result;
  buffer = (char*) malloc (sizeof(char)*lSize);
  for (size_t i=0;i<MAX_BRDS;i++)
    {
      cals[i]  = std::vector<Calibrations_t>(NCHN);
      for (size_t j=0;j<NCHN;j++)
        {
            cals[i][j] = calibrations_[i][j];
        }
    }
  return cals;
}

void TOF::BlobFileServer::ReadFromFile()
{

}

void TOF::BlobFileServer::RewindFiles()
{
  for (auto &k : blob_files_) 
    { if (k != nullptr) { fclose(k); } }

  blob_files_.clear();
  for (auto k : blob_filenames_)
    {
      std::cout << "[DEBUG] - Re-Opening file " << k << std::endl;
      blob_files_.push_back(fopen(k.c_str(), "rb"));
    }

}

/*******************************************************/

void TOF::BlobFileServer::SetNBoardsForRandomEvents(uint nboards)
{
   for (size_t k=0; k<nboards; k++)
   {current_event_number_.push_back(first_random_event_id_);}
}

/*******************************************************/

void TOF::BlobFileServer::GenerateBoardDNAs()
{
   if (random_board_dnas_.size() == nboards_) return;
   unsigned long long dna = 1000000000000000;
   for (size_t k=0; k<nboards_; k++) 
   {
       random_board_dnas_.push_back(dna);
       dna++;
   } 
}

/*******************************************************/

BlobEvt_t TOF::BlobFileServer::GenerateRandomEvent(uint rbid)
{
    BlobEvt_t event;
    //assert random_board_dnas_.size() >= nboards; 
    // fill fields here 
    event.head = 61680; 
    //event.status;
    //event.len;
    //event.roi;
    event.dna = random_board_dnas_[rbid];
    //event.fw_hash;
    //event.id; 
    //event.ch_mask;
    event.event_ctr = current_event_number_[rbid];
    //event.dtap0;
    //event.dtap1;
    //event.timestamp;
    //event.ch_head[NCHN];
    //event.ch_adc[NCHN][NWORDS];
    //event.ch_trail[NCHN];
    //event.stop_cell;
    //event.crc32;
    event.tail = 21845; 
    
    
    ++current_event_number_[rbid];
    return event;
}

/*******************************************************/

void TOF::BlobFileServer::Serve()
{ 
 
  // setup zmq
  std::string ip_addr = "tcp://127.0.0.1";
  int port = 38830;
  std::string address;
  std::cout << "[INFO] setting up sockets for " << nboards_ << " boards!" << std::endl;
  sockets_.reserve(nboards_);
  for (uint16_t k=0;k<nboards_;k++)
    {
      address = ip_addr + ":" + std::to_string(port);
      std::cout << "[INFO] using socket address " << address << std::endl;
      //std::unique_ptr<zmq::socket_t> sock = std::unique_ptr(new zmq::socket_t(zmq_context_, zmq::socket_type::req));
      zmq::socket_t sock = zmq::socket_t(zmq_context_, zmq::socket_type::req);
      sock.connect(address.c_str());
      std::cout << "[INFO] connected" << std::endl;
      sockets_.push_back(std::move(sock));
      //zsock_destroy(address.c_str());
      //sockets.push_back(zsock_new_req(address.c_str()));
      port += 1;
      std::string ping = "RB0" + std::to_string(k);
      zmq::message_t msg1(ping.c_str(), 4);
      sockets_[k].send(msg1, zmq::send_flags::none);
      // Fill a message passed by reference
      auto res = sockets_[k].recv(msg1, zmq::recv_flags::none);  
      std::cout << msg1.to_string() << std::endl;
    }
  // give time for the sockets to be setup
  sleep(5);

  std::chrono::time_point<std::chrono::system_clock> now, before;

  before = std::chrono::system_clock::now();
  now    = std::chrono::system_clock::now();
  std::chrono::duration<double> elapsed_seconds;
  // stop the broadcasting of events after final seconds
  std::chrono::duration<double> final(300);
  long lSize;
  //char * buffer;
  size_t result;
  //std::vector<unsigned char>* cachebuffer = new std::vector<unsigned char>(5000*1024);
  bool all_files_ended_sum = false;
  std::vector<bool> all_files_ended;//(false, nboards_);
  
  for (uint k=0; k<nboards_; k++) all_files_ended.push_back(false);

  long nevents_sent = 0;
  long npackets_sent = 0;

  // positions in the bytestream
  // the first per event,
  // the second for the global buffer
  unsigned char padding = 0; 
  size_t npadbytes = 90;
  uint nevents_cache = 1000; // simulate cache size
  uint sleep_time    = 2;  // simulate a certain event rate
                           // sleep_time in seconds 
                           // so the rate will be something like
                           // nevents_cache/sleep_time
  uint32_t BLOBEVENTSIZE = 36 + (NCHN*2) + (NCHN*NWORDS*2) + (NCHN*4) + 8;
  while (1)
    {
      std::cout << "newiter" << std::endl;
      for (int k=0;k<nboards_;k++) {
        // one cache per board
        // allocate the memory we need for 
        // nevents_cache*blobevent sizze
          std::cout << "[INFO] BlobEvt_t size " << sizeof(BlobEvt_t) << std::endl;
        std::vector<unsigned char>cachebuffer(nevents_cache* (BLOBEVENTSIZE + npadbytes),0);
        //for (uint k=0;k<npadbytes;k++) cachebuffer[k] = padding;
        std::cout << "==== NEW PACKET BOARD " << k << std::endl;
        for (uint j=0;j<nevents_cache;j++)
          {// lets use ReadEvent to get the packet length

            //if (inevent[k]==1 && status_[k] != -1 ) { // status=-1 -> EOF
            //std::cout << "[INFO] Reading events..." << std::endl;
            status_[k] = ReadEvent(blob_files_[k], &event_[k]);
            //std::cout << "[INFO] Read event success flag " << status_[k] << std::endl;
            if (status_[k] == -1) all_files_ended[k] = true;
            //if (status_[k] == -1) std::exit(1);
            lSize = event_[k].len;
            //std::cout << "[INFO] status, len, evt ctr, tail " <<  event_[k].status << " " << event_[k].len << " " << event_[k].event_ctr << " " << event_[k].tail << std::endl;

            // modify the event counter so that it just rises continously
            //event_[k].event_ctr += nevents_processed_ + j;
            
            //// FIXME - the data has bad event stati
            //event_[k].tail = 21845;
            if (j > 0)
                {encode_blobevent(&event_[k], cachebuffer, (j*BLOBEVENTSIZE + npadbytes )); }
            else { encode_blobevent(&event_[k], cachebuffer, 0); }
            //std::cout << "-- " << j << " --"  << std::endl;
            //std::cout << "CHECK " << event_[k].head << " " << event_[k].tail << std::endl;
            // add some padding after each event
            //std::cout << event_[k].head << "\n";
            //std::cout << event_[k].status << "\n";
            //std::cout << event_[k].len << "\n";
            //std::cout << event_[k].roi << "\n";
            //std::cout << event_[k].dna << "\n";
            //std::cout << event_[k].fw_hash << "\n";
            //std::cout << event_[k].id << "\n";
            //std::cout << event_[k].ch_mask << "\n";
            //std::cout << event_[k].event_ctr << "\n"; //printbinary(event.event_ctr, 32);
            //std::cout << event_[k].dtap0 << "\n";
            //std::cout << event_[k].dtap1 << "\n";
            //std::cout << event_[k].timestamp << "\n";
            //std::cout << "crc32 " << event_[k].crc32 << "\n";
            //std::cout << "stop_cell " << event_[k].stop_cell << "\n";
            //std::cout << "tail " << event_[k].tail << "\n";
            //padding
            //for (uint k=0;k<npadbytes;k++) cachebuffer.push_back(padding);

           //std::cout << status_[k] << std::endl;//
          } // end loop over events per board to emulate cache
          
          //std::cout << "[DEBUG] encoding finished!" << std::endl;
        
        //all_files_ended = (all_files_ended || (status_[k] == -1));
        //std::cout << " read " << result << std::endl;
        //std::cout << " cachebuffer->size() " << cachebuffer->size() << std::endl;
        //zframe_t *frame = zframe_new(cachebuffer.data(),cachebuffer.size());
        //if (frame == nullptr) 
        //{
        //    std::cerr << "[FATAL] allocating frame for zmq failed" << std::endl;
        //    std::exit(1);
        //}
        std::cout << "[INFO] We have sent " << npackets_sent -1 << " packets "  << std::endl;
        //&frame;
        //sockets_[k];
        //
        zmq::message_t msg(cachebuffer.data(),cachebuffer.size());
        //std::cout << "[DEBUG] we are sending a buffer of size " << cachebuffer.size() << std::endl;
        //std::cout << " -- raw binary data -- " << cachebuffer.data() << std::endl;
        zmq::message_t server_response("hello world!", 12);
        try { 
           sockets_[k].send(msg, zmq::send_flags::none);
           //sleep(0.1);
           auto res = sockets_[k].recv(server_response, zmq::recv_flags::none);  
           std::cout << server_response.to_string() << std::endl;
           // rest a bit to simulate a reasonable rate
           sleep(sleep_time);
        } catch (zmq::error_t exception) {
            std::cout << "[WARN] sending message failed " << std::endl;
        }
        //zframe_send(&frame, sockets_[k],0);
        //std::cout << "[DEBUG] frame send!" << std::endl;
        //zframe_destroy (&frame);
        //std::cout << "[DEBUG] frame destroyed!" << std::endl;
        npackets_sent++;
        //std::exit(1);
        //cachebuffer.clear();


        //std::cout << " zframe send " << std::endl;
        //std::cout << event_[k].len << std::endl;
        //sockets[k]

        now = std::chrono::system_clock::now();
        elapsed_seconds += now -before;
        before = now;
        //}
        //std::cout << "...";  
      }// end for loop over boards
      nevents_processed_+= nevents_cache;

      // print out/debug
      nevents_sent+= nevents_cache;
      if (nevents_sent % 5 == 0)
        {
          std::cout << " -- Sent " << nevents_sent*nevents_cache << " events!" << std::endl;
          for (uint k=0;k<nboards_; k++) 
            std::cout << " -- last eventcounter " << k <<  event_[k].event_ctr << std::endl;
        }
       

      std::cout << "[DEBUG] EOF checks!" << std::endl;
      //break;
      all_files_ended_sum = true;
      for (bool eof : all_files_ended) std::cout << eof << std::endl;
      for (bool eof : all_files_ended) all_files_ended_sum = eof && all_files_ended_sum;
      if (n_seconds_finish_criterion_ > 0)
      {
        if (elapsed_seconds > final)
         {
           std::cout << "[INFO] we reached the maximum run time, so we'll stop reading events!" << std::endl;
           break;
         }
      } 
      if (n_events_finish_criterion_ > 0) 
        {
          if (nevents_sent >= n_events_finish_criterion_)
            { std::cout << "[INFO] reached number of events required!" << std::endl;  
              break;}
        }
      if (all_files_ended_sum) 
        {
          std::cout << "[INFO] files have ended" << std::endl;
          if (files_on_repeat_) RewindFiles();
          else break;
        }
    } // end while-true loop
}

void TOF::BlobFileServer::SetFilesOnRepeat(bool make_it_so)
{
    files_on_repeat_ = make_it_so;
}

void TOF::BlobFileServer::SetFinishAfterNEvents(uint32_t n_events)
{
  n_events_finish_criterion_ = n_events;
}

void TOF::BlobFileServer::SetFinishAfterTime(uint32_t n_seconds)
{
  n_seconds_finish_criterion_ = n_seconds;
}
