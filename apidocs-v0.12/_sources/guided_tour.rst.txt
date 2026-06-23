A guided tour to get you started
================================

.. note:: Data are stored (if not locally available to you) on 
          the `Hawaii nextcloud system <https://uhams02a.phys.hawaii.edu/nextcloud/index.php>`_ 

Data formats
------------

GAPS employs a various number of data formats. There are

* `.bin`     - (commonly called "binary" files (even though all GAPS data is 
  stored in "binary" format)) - These files get written by the GSE system.
  These files contain everything sent to ground. 
* `tof.gaps` - datafiles written by this library (`gondola/liftof`). These are 
  used throughout the TOF system. This is a highly optimzied data format which
  allows to store the waveforms of the TOF system. This data will include 
  monitoring data from the tof system.
* `.gaps`    - datafiles written by this library, employing the `caraspace` system, 
  which allows to merge the above `.bin` and `.tof.gaps` files in a very efficient 
  way. This introduces a "vertical" merging, meaning that events with the same id from 
  either of the streams will be stored together within a so-called "frame". 
  For all GAPS rundata, `.gaps` files will be created as part of the data processing. 
  Since these data are not calibrated, these are called "L0" data. 
* `.root`    - data with CERN's widely used "ROOT" library. These files typically 
               contain reconstructed event data, where one or more of several different 
               reconstruction algorithms are applied and are typically written throughout
               the `SimpleDet library <https://uhhepvcs.phys.hawaii.edu/philipvd/SimpleDet>_`. 

How to read the data
--------------------

L0 data - binary merger of telemetry and TOF disk data
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For reading the L0 data, you can do the following :: 

  import gondola 
  # Set up a reader - For L0 data, we will need "CRReader" 
  reader = CRReader("/path/to/L0/data") # <- can be a single file 
                                        # or a directory, it will
                                        # figure it out automatically 

  # The reader can count the number of frames in the files 
  # not every frame will correspond to event data, there is 
  # also monitoring  
  nframes = reader.count_frames() 

  # The reader acts as an iterator and can be looped over. 
  import tqdm # <- Just for the progressbar, can be omitted 
  for frame in tqdm.tqdm(reader, total=nframes):
      # the frame has an "index" showing the contents of the 
      # frame. It is a dictionary string -> packet 
      print (frame.index) 
      # This allows to check if a frame contains a certain key 
      if frame.has('TelemetryPacketType.NoGapsTriggerEvent'): 
          ev = frame.get_telemetryevent() # no argument needed if this 
                                            is unambiguous 
          # if multiple events are in the frame, specify packet name 
          ev = frame.get_telemetryevent('TelemetryPacketType.NoGapsTriggerEvent') 
          ev.tof # <- packet TOF data for Telemetry 
          ev.tracker # <- tracker hits 

          ... 

Telemetry data (".bin") files 
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

lorem ipsum 

.. admonition:: Different readers
   For the 3 types of data mentioned above, (except the `.root` data), currently
   there are 3 different readers implemented. `CRReader`, `TofPacketReader` and 
  `TelemetryPacketReader`, all working in a similar fashion.  


Geometry & Calibration data are stored in a built-in database! (How to access them) 
-----------------------------------------------------------------------------------

`gondola` utilizes an internal sqlite database, which is automatically installed with the library. In fact, upon importing the 
`gondola library, it should greet you with somehting like 

.. code-block:: python 

   Welcome to gondola v0.11.11, a software suite for the 🎈 GAPS experiment! Bulld for 🐍 with the power of 🦀! ✨
   -- The database has been set to GONDOLA_DB_URL /srv/gaps/gaps-online-software/gondola-test/gondola-test/.venv/lib/python3.13/site-packages/gondola/gaps_flight.db  

which should tell you the path to the sqlite database file. 

With the help of this database, you can actually do the following:

.. code-block:: python 

   import gondola as gon 
   paddles = gon.db.TofPaddle.all_as_dict() # dictionary of paddle id -> TofPaddle 
   print(paddles[2])
   -----> 
     <TofPaddle: <TofPaddle:
  ** identifiers **
     pid                : 2
     vid                : 110000100
     panel id           : 1
    ** connedtions **
     DSI/J/CH (LG) [A]  : 2  | 3 | 09
     DSI/J/CH (HG) [A]  : 2  | 3 | 05
     DSI/J/CH (LG) [B]  : 2  | 3 | 10
     DSI/J/CH (HG) [B]  : 2  | 3 | 06
     RB/CH         [A]  : 16 | 5
     RB/CH         [B]  : 16 | 6
     LTB/CH        [A]  : 08 | 9
     LTB/CH        [B]  : 08 | 10
     PB/CH         [A]  : 04 | 9
     PB/CH         [B]  : 04 | 10
     MTB Link ID        : 15
     cable len [cm] :
      ↳ 330.00
      (Harting -> RB)
     cable times [ns] (JAZ) :
      ↳ 13.76 13.56
    ** Coordinates (L0) & dimensions **
     length, width, height [mm]
      ↳ [180.00, 16.00, 0.63]
     center [mm]:
      ↳ [67.09, 0.00, 110.39]
     normal vector:
      ↳ [0.00, 0.00, 1.00]
     A-side [mm]:
      ↳ [67.09, 90.00, 110.39]>
     B-side [mm]:
      ↳ [67.09, -90.00, 110.39]>>

  # also in a similar fashion for TrackerStrips 
  print(gon.db.TrackerStrip.all_as_dict()[0])
  <TrackerStrip: <TrackerStrip [0]:
     vid                : 200050200
     layer              : 0
     row                : 0
     module             : 0
     channel            : 0
     strip center [mm]:
      ↳ [-58.22, 54.56, 98.18]
     detector (disk) center [mm]:
      ↳ [-54.56, 54.56, 98.18]
     strip principal direction:
      ↳ [0.00, 0.00, 0.00]>

.. admonition :: Interacting with SimpleDet? 

   There are 2 special functions in `gondola.db` which produce maps of volume id <-> hardware (the "real" id):
   `gon.db.get_hid_vid_map()` for hardware id -> volume id and `gon.db.get_vid_hid_map()` for 
   volume id -> hardware id.  In this context, the hardware id is the strip id (LRMMSS) for the tracker and the paddle id
   for the TOF (1-160).

The database contains much more, there are for example it also contains `TrackerStripPedestal`, `TrackerStripTransferFunction` 
and `TrackerStripMask` which are relevant for tracker calibrations. 


TOF timings, time-of-flight and beta calculations
-------------------------------------------------

Before this gets discussed in terms of software, conceptionally beta and the time-of-flight itself
need to be discussed. First, in order to obtain them, some kind of reconstruction is necessary, 
even if that is just a line between two points. However, this already assumes something about 
the event topology. For complex events, with many hits and maybe several competing (coincident) 
tracks, obtaining a simple and meaningful time-of-flight is outside the scope of this 
software documentation.

To illustrate what the software is capable though, we assume a single track sample, where the 
primary only leaves deposited energy in detectors it crosses (e.g. no emission of delta electrons 
which could then cause secondary hits). While this is a somewhat ideal case, it is pretty much 
realized with muon data, as obtained in test campaigns on the ground. In this case, a good 
proxy for the time-of-flight can be obtained fairly simply with the `gondola` software package. 

.. note::  The time-of-flight here is defined by the subtraction of the first hit on the outer TOF from the first hit on the inner TOF

Exapmle:

```
# before an accurate TOF can be calculated, some setup is needed 
t_offsets   = dict() # these are externally calculated timing constants 
                     # which absorb unknown systematics and have been 
                     # calculated by paddle 

# different iterations of these constants can be loaded
offsets     = go.db.TofPaddleTimingConstant.as_dict_by_name("GraceV2")
# the individual values in this dictionary contain more inforrmation, 
# we can just strip them down to be easilier to handle.
for k in offsets:
    t_offsets[k] = offsets[k].timing_constant

# let's assume you have some binary data (.bin format) 
fname  = '/path/to/binary/RAW000000_000000.bin' 
reader = go.io.TelemetryPacketReader(fname)
for pack in reader:
    if pack.is_event_packet:
        # as mentioned in the above excerpt, we focus on "clean tracks" for now 
        if pack.header.packet_type != go.packets.TelemetryPacketType.NoGapsTriggerEvent:
            continue

        ev = go.events.TelemetryEvent.from_telemetrypacket(pack)
        # this is an entirely seperate topic, however, for here we can only accept hits 
        # which have sane hits in both paddle ends
        bad = ev.tof_remove_non_causal_hits() # we can store bad hits to be analyzed later 

        # apply the systematic offsets 
        ev.set_tof_timing_offsets(t_offsets)
        # this step is critical. Since the correct calculation of the hit times 
        # depends on the channel 9 phase difference between the hits, this has 
        # to be happening on the event leve. As a side effect, the first hit time 
        # in the event gets set to zero. This call is also included in the tof_time_of_flight
        # function, however, if beta is calculated manually, it has to be applied!
        ev.tof_normalize_hit_times() 
        # then there is a one-shot function for the tof, which also returns 
        # the calculated beta value as well as other variables which went into 
        # the calculation for some diagnostics
        tof, beta, phase, distance, harting_cable_time_difference = ev.tof_time_of_flight 

        # alternatively, beta can be calculated manually 
        hits = ev.tof.hits 
        outer = [h for h in hits if h.paddle_id > 60]
        inner = [h for h in hits if h.paddle_id < 61]
        if len (outer) > 0 and len(inner) > 0:
            outer  = sorted(outer, key = lambda h: h.event_t0) 
            inner  = sorted(inner, key = lambda h: h.event_t0) 
            t_diff = inner[first].event_t0 - outer[first].event_t0
            dist   = inner[first].distance(outer[first])/1000.0
            beta   = 0
            if t_diff != 0:
                beta = dist/(t_diff*1e-9)/299792458.0;
``` 

Calibrations
------------

TOF
^^^ 

Readoutboard calibrations can be loaded directly from the files with the following routine 

.. code-block:: python 

   import gondola as gon 
   from pathlib import Path 
   # creates a dictionary of rb_id -> RBCalibrations 
   calib = gon.calibration.load_rb_calibrations(Path('/path/to/your/calibration-directory-with-all-calibration-files'))
   

Tracker
^^^^^^^


Monitoring data ("Housekeeping")
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

* TOF monitoring data 

* Liftof settings 

The general (program) settings for the liftof code as it is running on the instrument, are telemetered down at 
the start of each new run. They are bytecompressed and 
so the bytestream of this packet needs specials treatement.

.. code-block:: python 

  #! /usr/bin/env poython 
  
  import gondola as gon 
  import time
 
  start_of_run_time = 3600 # e.g. for a run that started and hour in the past

  files  = gon.io.grace_get_telemetry_binaries(time.time() - start_of_run_time, time.time(), '/prestaging/live/')
  for f in files:
      reader = gon.io.TelemetryPacketReader(f)
      for pack in reader:
          if pack.packet_type == gon.packets.TelemetryPacketType.AnyTofHK:
              tp = gon.packets.TofPacket.from_bytestream(pack.payload, 0)
              if tp.packet_type == gon.packets.TofPacketType.LiftofSettings:
                  print (tp)
                  lpt = tp
                  break
  # this is the file decompression, and the config 
  # will be saved in test.toml
  gon.io.decompress_toml(lpt.payload, 'test.toml')


gaps.live
=========

