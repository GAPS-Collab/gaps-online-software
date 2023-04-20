v0.4.0
* Waveform analysis possible directly on the board
  A new `--analyze` flag will take that into account
* Simplified --vcal, --tcal, --noi flags for calibration

v0.3.4
* New feature, random self trigger mode with 
  randomly space triggers in time.

v0.3.3
* Various bugfixes, code cleanup
* Fixes issue with testing mode with self-trigger

v0.3.0
* Changed the runner - the runner will now read out 
  the buffers as well. This saves an entire thread.
* The buffer handler determines the active buffer by 
  checking dma_ptr. 
* These should require significanlty less checking of
  the registers and usage of less CPU resources.
* slight API change/WIP
* introduces CTRL+C to stop data taking (disables triggers
  and breaks the infinite loop in main)

v0.2.3
* Changed the buffer readout model to hopefully be more 
  reliable and resourceful

v0.2.2
* Reworked the command routine and the handling of RunParams

v0.2.1

* Changed 0MQ scheme. RBs will prefix their packets with "RBXX"
* 0MQ command channel is now SUB


