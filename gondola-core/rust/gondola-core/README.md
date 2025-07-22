# Events

* MergedEvent     - TofEventSummary + Tracker
* TofEvent        - TofEventSummary + RB meta information + waveforms
* TofEventSummary - No waveforms
* CaliEvent       - Only hits and minimal information TOF and tracker, 
                    tracker hits masked and transfer function applied

# I/O system

* TelemetryPacketReader   -> Reads .bin files from gse
* TofPacketReader/Writer  -> Reads/writes .tof.gaps files
* CaraspcaeReader/Writer  -> Reads/writes .gaps files


# New calibrated event

Event
* event status
* event time
* trigger type
* interesting
* ntrigger hits
Hit 
* x,y,z,t,hardware id, energy

# Performance improvements

-> Database system: switch from django to pure-rust implementation with diesel + pyo3
   This gives a performance boost by 3x for get_tofpaddles():
   Rust : 500mic sec, django 1.5 milli sec
 
