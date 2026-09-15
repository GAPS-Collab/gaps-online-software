#! /usr/bin/env python 


"""
Add TOF paddle temps in simplified form to the gondola database
"""
import tqdm
import gondola as go

# check gondola version
GON_VERSION_REQUIRED = '0.12.31' 
if not go.version_at_least(GON_VERSION_REQUIRED):
    print(f'ERROR - got version {go.get_version()} but need version {GON_VERSION_REQUIRED}')
    raise ImportError("gondola needs to be at least version {GON_VERSION_REQUIRED}!")

if __name__ == '__main__':
    
    import argparse
    from pathlib import Path

    parser      = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    args = parser.parse_args()
    # maps 
    map_a, map_b = go.db.get_rbid_pbchannel_pid_map()

    files = args.input_dir.glob('*.bin')
    files = [str(k)for k in sorted(files)]
    print (f'-> Found {len(files)} files for input!')
    pamoniseries = go.monitoring.PAMoniDataSeries()  
    pamoniseries.max_size = int(1e8)
    for f in tqdm.tqdm(files):
        pamoniseries.add_telemetryfile(f)
    sizes = pamoniseries.sizes
    first_ts = pamoniseries.first_ts
    pdl_temps = []
    for b in tqdm.tqdm(pamoniseries.boards, desc='Loading moni data...'):
        for k in range(sizes[b]):
            pm = pamoniseries.get_monidata_for_board(b,k)
            pid_temps = pm.get_pid_temps(map_a,map_b)
            #print (pm)
            #print (f'Timestamp {first_ts + pm.timestamp}')
            for k in pid_temps:
                tpt   = go.db.TofPaddleTemp() 
                tpt.paddle_id = k
                tpt.utc_timestamp = first_ts + pm.timestamp
                tpt.temp_a = pid_temps[k][0] 
                tpt.temp_b = pid_temps[k][1]
                #print (tpt)
                pdl_temps.append(tpt)    
        print (f"Inserting {len(pdl_temps)} rows in db!")
        go.db.create_tof_paddle_temp_table('../gaps_flight.db', pdl_temps)
        pdl_temps = []
            #break
