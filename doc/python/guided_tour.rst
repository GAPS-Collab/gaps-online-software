A guided tour to get you started
================================

.. note:: How to get data
          Data are stored (if not locally available to you) on 
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

