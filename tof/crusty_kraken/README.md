### Alternative TOF server software

This is an alternative version for the offical tof server software,
its tasks are the following:

* Connect to tof readoutboards through the network and gather their 
  binary blob data
* Waveform analysis to extract timing and charge information
* Assembling the events across different readout boards
* Packaging this information into a binary format
* Sending the packaged information elsewhere

