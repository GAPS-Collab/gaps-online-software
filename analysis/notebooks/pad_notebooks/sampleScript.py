1
import gondola as gon
import time

import channel_rates

starttime = 1766959800
endtime =   1766979800
files = gon.io.grace_get_telemetry_binaries(
    starttime, #start 1765835400
    endtime, #1766949800     1767039800 like 22 hours...
    #, #end time 1767979800 (testing it is for the random 100,000 seconds of flight)
    '/home/gaps/tof-data/antarctica/nextcloud/flight_2025-26'
)

fileCount = 0
for i in range(0, len(files), chunk_size):
    print(i)
    print("chunk^")
    chunk = files[i:i+chunk_size]
    
    pa   = gon.monitoring.PAMoniDataSeries()
    cpuM = gon.monitoring.CPUMoniDataSeries()
    rbM  = gon.monitoring.RBMoniDataSeries()
    ltbM = gon.monitoring.LTBMoniDataSeries()
    mtbM = gon.monitoring.MasterTriggerHBSeries()
    sipM = gon.monitoring.SipPosMoniDataSeries()
    pbM  = gon.monitoring.PBMoniDataSeries()
    
    for m in [pa, cpuM, rbM, ltbM, mtbM, sipM]:
        m.max_size = int(1e7)
    
    
    for f in chunk:
        #monitoring information
        sf = str(f)
        pa.add_telemetryfile(sf)
        cpuM.add_telemetryfile(sf)
        rbM.add_telemetryfile(sf)
        ltbM.add_telemetryfile(sf)
        mtbM.add_telemetryfile(sf)
        sipM.add_telemetryfile(sf)
        pbM.add_telemetryfile(sf)