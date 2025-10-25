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

.. admonition:: Different readers
   For the 3 types of data mentioned above, (except the `.root` data), currently
   there are 3 different readers implemented. `CRReader`, `TofPacketReader` and 
  `TelemetryPacketReader`, all working in a similar fashion.  


Database access 
---------------

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

Calibratons
-----------

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


gaps.live
=========

