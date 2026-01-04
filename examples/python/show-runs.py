#! /usr/bin/env python

import tomllib
import tqdm
import argparse

from glob import glob
from pathlib import Path
from datetime import datetime, timezone, UTC

from rich.console import Console
from rich.table import Table

import gaps_online as go
#from gaps_online.rust_api.io import TofPacketReader
#from gaps_online.rust_api.moni import MtbMoniData
TofPacketReader = go.rust_api.io.TofPacketReader
MtbMoniData     = go.rust_api.moni.MtbMoniData

import re

#pattern.search(f).groupdict()

path = Path('/home/gaps/csbf-data')
files = glob('*.toml')
files = sorted(files)

def get_directory_size(directory: Path) -> int:
    return sum(f.stat().st_size for f in directory.rglob('*') if f.is_file())

def get_ts_from_toffile(fname):
    pattern = re.compile('Run[0-9]*_[0-9]*.(?P<tdate>[0-9_]*)')
    ts = pattern.search(str(fname)).groupdict()['tdate']
    ts = datetime.strptime(ts, '%y%m%d_%H%M%S')
    ts = ts.replace(tzinfo=timezone.utc)
    return ts


def get_trigger_info(t):
    prescale = t['mtb_settings']['trigger_prescale']
    ttype    = t['mtb_settings']['trigger_type']
    tsup     = t['mtb_settings']['trace_suppression']
    #print (f'{ttype} - {prescale:4.2f}')
    return ttype, prescale, tsup

def get_run_meta(f):
    t = tomllib.load(open(f, 'rb'))
    ttype, prescale, tsup = get_trigger_info(t)
    pattern = re.compile('run(?P<runid>[0-9]*).toml')
    runid = pattern.search(str(f)).groupdict()['runid']
    runid = int(runid)
    return { runid : {'trigger_type'     : ttype,
                      'prescale'         : prescale,
                      'trace-suppressed' : tsup,
                      'rate'             : None,
                      'runtime'          : None,
                      'begin-time'       : None,
                      'size'             : None}}

# all these runs have .toml files living in the main directory
runinfo  = dict()
for k in tqdm.tqdm([j for j in path.glob('*')]):
    ts = None
    runtime = None
    rate = None
    if str(k).endswith('.toml'):
        pattern = re.compile('run(?P<runid>[0-9]*).toml')
        runid = pattern.search(str(k)).groupdict()['runid']
        runid = int(runid)
        rundir = str(k).replace(k.name,'')
        size = get_directory_size(Path(rundir))
        size = int(size)/1e9
        if size == 0:
            print (k, 'has zero size')
            continue
        runinfo.update(get_run_meta(k))
        # go into the associated directory 
        runinfo[runid]['size'] = f'{size:.2f}G' 
        rundir = Path(str(k).replace(k.name,str(runid)))
        fname = sorted([j for j in rundir.glob('*.tof.gaps')])
        if fname:
            fname_last = fname[-1]
            fname = fname[0]
            reader = TofPacketReader(str(fname), filter=go.rust_api.io.PacketType.MonitorMtb)
            for pack in reader:
                moni = MtbMoniData()
                moni.from_tofpacket(pack)
                rate = moni.rate
                break
            ts = get_ts_from_toffile(fname)
            ts_last = get_ts_from_toffile(fname_last)
            dur = ts_last - ts
            runtime = dur.total_seconds()/3600
            runtime = f'{runtime:.3f}h'
            ts = ts.strftime('%y/%m/%d %H:%M:%S UTC')
        runinfo[runid]['begin-time'] = ts 
        runinfo[runid]['runtime']    = runtime
        runinfo[runid]['rate']       = rate

for k in tqdm.tqdm([j for j in path.glob('*')]):
    ts = None
    runtime = None
    rate = None
    if k.is_dir():
        #runid = int(k.split()[1])
        size = get_directory_size(k)
        size = int(size)/1e9
        if size == 0:
            continue
        try:
            runid = int(k.name)
        except ValueError as e:
            print (f'-- -- Unable to extract runid from {k.name}')
            continue
        
        if runid in runinfo.keys(): 
            # in that case, we already have parsed 
            # the toml file
            runinfo[runid]['size'] = f'{size:.2f}G' 
            fname = sorted([j for j in k.glob('*.tof.gaps')])
            if fname:
                fname_last = fname[-1]
                fname = fname[0]
                reader = TofPacketReader(str(fname), filter=go.rust_api.io.PacketType.MonitorMtb)
                for pack in reader:
                    moni = MtbMoniData()
                    moni.from_tofpacket(pack)
                    rate = moni.rate
                    break
                ts = get_ts_from_toffile(fname)
                ts_last = get_ts_from_toffile(fname_last)
                dur = ts_last - ts
                runtime = dur.total_seconds()/3600
                runtime = f'{runtime:.3f}h'
                ts = ts.strftime('%y/%m/%d %H:%M:%S UTC')
            runinfo[runid]['begin-time'] = ts 
            runinfo[runid]['runtime']    = runtime
            runinfo[runid]['rate']       = rate
        else:
            try:
                runinfo.update(get_run_meta(f'{k}/run{runid}.toml'))
            except:
                print(f'-- --Unable to find toml file for {runid}')
                continue
            fname = sorted([j for j in k.glob('*.tof.gaps')])
            if fname:
                fname_last = fname[-1]
                fname = fname[0]
                reader = TofPacketReader(str(fname), filter=go.rust_api.io.PacketType.MonitorMtb)
                for pack in reader:
                    moni = MtbMoniData()
                    moni.from_tofpacket(pack)
                    rate = moni.rate
                    break
                ts = get_ts_from_toffile(fname)
                ts_last = get_ts_from_toffile(fname_last)
                dur = ts_last - ts
                runtime = dur.total_seconds()/3600
                runtime = f'{runtime:.3f}h'
                ts = ts.strftime('%y/%m/%d %H:%M:%S UTC')
            runinfo[runid]['begin-time'] = ts 
            runinfo[runid]['runtime']    = runtime
            runinfo[runid]['rate']       = rate
            runinfo[runid]['size'] = f'{size:.2f}G' 
    #else:
    #    runinfo[runid]['begin-time'] = None 
    #    runinfo[runid]['runtime']    = None
    #    runinfo[runid]['rate']       = None




console = Console()
table = Table(title="Run overview")
table.add_column("Run Id"     , style="cyan"   , justify="left")
table.add_column("Trigger"    , style="magenta", justify="left")
table.add_column("Prescale"   , style="green", justify="left")
table.add_column("Rate"       , style="green", justify="left")
table.add_column("Trace supp.", style="green", justify="left")
table.add_column("Data size"  , style="green", justify="left")
table.add_column("Start time" , style="green", justify="left")
table.add_column("Duration (approx)", style="green", justify="left")

for run in sorted(runinfo.keys()):
    table.add_row(
        str(run),
        str(runinfo[run]["trigger_type"]),
        f'{runinfo[run]["prescale"]:.4f}',
        str(runinfo[run]["rate"]),
        str(runinfo[run]["trace-suppressed"]),
        str(runinfo[run]["size"]),
        str(runinfo[run]["begin-time"]),
        str(runinfo[run]["runtime"])
    )

# Print the table to the console
console.print(table)

