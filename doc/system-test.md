# System testing

### Legend:

**CRITICAL ITEMS - WITHOUT THESE WE HAVE A HARD TIME TO FLY**

_Comment_

Test types:

_All test results except unit tests need to be checked by a person_


Unit Test: Automatic software test

| `type`      | `operator`  |  `comment`      |    `explanation` | 
|-------------|-------------|-----------------|------------------|
| on-site     | [human]     |  test/debugging |   will require time with the system and a person |
| on-site     | [automatic] |  test/debugging |   will require time with the system, but can be done without supervision. |
| remote-site | [human]     |  test/debugging |   will require some remote access to some part of the system, e.g a readoutboard |
| off-site    | [human]     |  test/debugging |   no connection to any system required, but needs to be performed by a person |
| off-site    | [unit]      |  test/debugging |   done by software. Only failures reported.


`on-site` test might depend on flight network, switches hardware etc.)

`remote-site`  can be done without the *exact* flight hardware, only the item subject to the 
test needs to be of flight status

`off-site` is typically a software test which can be done with simulated/emulated participants

### System testing is defined as completly successful if:

* At least 24h run with cold tracker without failure
* Remote calibration procedure performed multiple times
* System passed critical scenario test (e.g. loss of connection, power outage, dead RB
* Run can be interupted/resumed from ground
* Monitoring data from all components received at ground


## Data pipeline :

Flow direction downstream:

* MasterTrigger
* ReadoutBoard (TOF) - Sili Modules (Tracker)
* TofComputer        - Tracker Daq
* Flight Computer
* Ground systems

Each system has to report readiness.
Readiness is defined for each system as follows:

* MasterTrigger

- *IMPLEMENTATION OF TRIGGER ALGORITHM*         -> remote site [human] development
- Connection test (UDP) to Tof Computer for 24 hours (arbitrary time) 
  without losing/skipping any event.            -> remote-site [automatic] test/debugging
- Connection test via dedicated wire to RB/Tracker DAQ _this is basically happening automatically when we do other test, so that is why it is "on-site"_ -> on-site [human] test/debugging
- *CHANNELMASK/HITMASK MUST BE IMPLEMENTED/WORKING* - _this mask identifies the ReadoutBoards which have participated in the trigger_
- Absolute timing: GPS connection. _This seems less critical, for analysis related altitude measurement seems more important_

* ReadoutBoard

- Write/read registers reliably     -> remote-site [human] test/debugging
- Deterministic/Fixed start up time -> remote-site [human] test/debugging
- Streaming of events in continuous (`StreamAnyEvent`) mode without skipping events -> remote-site [automatic] test/debugging
- Time between event request and served event in `RequestEvent` mode -> remotes-site [automatic] test/debugging
- Thread revivability                        -> remote-site/off-site [human] test/debugging
- Implementation of crucial commands on RB   -> off-site [human] development
- Testing of crucial commands on RB          -> remote-site [human] test/debugging
- Testing of crucial commands from ground    -> on-site [human] test/debugging
- Implementation of auxiliary commands on RB -> off-site [human] development
- Testing of auxiliary commands on RB        -> remote-site [human] test/debugging
- Testing of auxiliary commands from ground  -> on-site [human] test/debugging
- Sending monitoring data                    -> remote-site [human] test/debugging
- Calibration                                -> off-site [human] development
- Automatic calibration                      -> off-site [human] development
- Reporting of calibration results           -> off-site [human] development
- Software benchmarking, power/heat          -> remote-site [human] test/debugging
- Rate test - max rate before breakdown      -> on-site [human] test/debugging

* Sili Modules/Tracker DAQ

- Timing/Timestamps: compatible timestamps with tof -> on-site [human] test/debugging
- Eventid from master trigger,no missed event id    -> on-site [human] test/debugging
- Channel mapping/geometry -> off-site [human] development
- Reasonable length of busy signal, optimization    -> on-site [human] test/debugging 
- No Christmas light event in 7 day run             -> on-site [human/automatic] test/debugging
- **NO SHIFT OF EVENT ID BETWEEN TRACKER AND TOF EVER** -> on-site [human] analysis

* Tracker general

- Transfer functions for every module -> off/on-site [human] development
- Automated calibration procedure     -> on-site [human] test/debugging

* Tof Computer

- Stable connection to all RBs -> on-site [human/automatic] test/debugging
- Software event caches at 20% after 24h run -> on-site [human/automatic] tset/debugging
- No memory leak/corruption EVER -> on-site [human] test/debugging
- No loss of RB in events -> on-site [human] analysis
- Gathering of significant sample of lucky events ("4 leaf clover") with exactly 4 hits in the tof on overlapping paddles -> on-site [human] operations _this basically defines the length of our minimum test run. Might be at least 24hours._ -> on-site [human] analysis

* Flight Computer

- Gathering monitoring data from flight subsystems -> off-site [human] development
- Event merging Tof/Tracker                        -> on-site [human] test/debugging
- Maximum rate when system fails                   -> on-site [human] test/debugging
- Number of missed packets from the tof computer   -> on-site [human] test/debugging
- Interesting event search                         -> off-site [human] development
- Observing interesting events                     -> on-site [human] analysis
- Changing parameters trigger/interesting events   -> on-site [human] test/development 
- Receiving/Response to commands                   -> on-site [human] test/debugging
- Uploading of code/Update meechanism              -> off/on-site [human] concept
- Revivability/Reviving other systems              -> on-site [human] test/debugging
- Loss of connection to ground/reconnection        -> on-site [human] test/debugging
- Distribution of GPS clock, subsystem clock sync  -> on-site [humna] test/debugging
- Waveform request                                 -> off/on/remote-site [human] development 

* Ground computer 

- Reasonable system interface, operational by non expoert -> off-site [human] development
- Necessary plots defined and available                   -> off-site [human] development
- Sending of commands, receiving acknowledgement          -> off-site [human] development
- Archival of final data product                          -> off-site [human] development 
- analysis of system test data                            -> off-site [human] development
    
