#ifndef BLOBFILESERVER_H_INCLUDED
#define BLOBFILESERVER_H_INCLUDED

#include "TOFCommon.h"
#include "blobroutines.h"
//#include "tof_dataclasses.h"

#include <memory>

#include "czmq.h"
#include "zmq.hpp"

#include <cassert>

namespace TOF {

/**
 * Reads Blob data and spits it out event-by-event
 *
 *
 */
class BlobFileServer{
  public:
    BlobFileServer(std::vector<std::string> blobfilenames);
    ~BlobFileServer();
    BlobEvt_t* GetNextEvent();
    /**
     * By design, what GetNextEvent returns is an array of 
     * size nboards. Not all of these boards participate in 
     * the event though. The boolean mask tells which boards
     * participate
     *
     */
    std::vector<int> GetNextEventMask();

    /**
     * 
     * 
     */
    void ReadFromFile();

    void Serve();

    bool* BoardHasCalibration();
    std::vector<std::vector<Calibrations_t>> GetCalibrations();
    bool EventIdSanityCheck(BlobEvt_t event[],
                            int inevent[],
                            unsigned int&  eventid,
                            long& min_event,
                            long& max_event);

    BlobEvt_t GenerateRandomEvent(uint rbid);

    void GenerateBoardDNAs();

    void SetNBoardsForRandomEvents(uint nboards);

    void SetFilesOnRepeat(bool);

    void SetFinishAfterTime(uint32_t n_seconds);
    
    void SetFinishAfterNEvents(uint32_t n_events);

  private:

    void RewindFiles();

    void LoadCalibrations();
    struct Calibrations_t calibrations_[MAX_BRDS][NCHN];
    bool board_has_cal_[MAX_BRDS] = { 0 }; // init all to False

    std::vector<std::string> blob_filenames_;
    std::vector<FILE*> blob_files_;
    uint16_t nboards_;
    std::vector<int> board_in_event_mask_; // basically "inevent"
    Times_t times_;
    BlobEvt_t event_[MAX_BRDS];
    // we need this since we have to 
    // "hold" one event during the 
    // readout loop, so that we can 
    // return it. 
    // There might be a smarter way
    BlobEvt_t event_buffer_[MAX_BRDS]; 
    int status_[MAX_BRDS];
    //zmq_socket(MyTOFParam->context, ZMQ_REP);
    // we have one socket per each emulated readout board
    //void *context = zmq_ctx_new ();
    //std::vector<std::unique_ptr<zmq::socket_t>> sockets_;
    std::vector<zmq::socket_t> sockets_;
    zmq::context_t zmq_context_;

    unsigned int nevents_processed_;

    //! A counter which is used to make sure we don't emit
    //  the same eventid twice
    std::vector<unsigned int> current_event_number_;

    //! Randomly generated board dnas for random mode
    std::vector<unsigned long long> random_board_dnas_;

    //! The first event id for the random event generator
    unsigned int first_random_event_id_;

    bool files_on_repeat_;

    uint32_t n_events_finish_criterion_;
    uint32_t n_seconds_finish_criterion_;
};

} // end namespace
#endif
