# Licensed under a 3-clause BSD style license - see PYFITS.rst

import numpy as np
import tofpy

from .parsing import cleanSpikes, load

__all__ = ['calibrateBoard',
           'voltageCalibration','timingCalibration','applyVCal','applyTCal']

def calibrateBoard(gbf=None,gbf2=None,gbft=None,dv=0,vcalfile='',edge='average',
                   name='',npy=False):
    '''calibrateBoard: calibrate readout board/DRS4

    Parameters
    ----------
    gbf: vcal gbf (or .dat file name)
    gbf2: second vcal gbf (or .dat file name)
    gbft: tcal gbf
    dv: voltage difference gbf2 - gbf, in mV
    vcalfile:
    edge: edge argument for timingCalibration - rising, falling, or average
    name: calibration file name
    npy: option to also save calibration in npy format
    '''
    # vcal
    if vcalfile != '':
        cal = loadCalibration(vcalfile)
        vcal = cal[:3]
        # allow gbf arg to be used instead of gbft, when vcalfile is used
        if gbft is None and gbf is not None:
            gbft = gbf
    else:
        vcal = voltageCalibration(gbf,gbf2=gbf2,dv=dv)
    # tcal
    if gbft is None:
        tcal = None
    else:
        tcal = timingCalibration(gbft,vcal,edge=edge)
    # save
    if name == '':
        txtfile = 'rb{0}_cal.txt'.format(gbf.rbFromDNA())
    else:
        txtfile = name + '.txt'
    saveCalibration(vcal,tcal,txtfile=txtfile,npy=npy)
    
    
def saveCalibration(vcal,tcal,txtfile,npy=False):
    nchan = vcal.shape[1] # usually 8 or 9
    tracelen = vcal.shape[2] # usually 1024
    with open(txtfile,'w') as fp:
        for ch in range(nchan):
            for caltype in range(4):
                for i in range(tracelen):
                    if caltype == 0:
                        fp.write('{0:d}'.format(int(vcal[caltype][ch][i])))
                    elif caltype == 1:
                        fp.write('{0:.1f}'.format(vcal[caltype][ch][i]))
                    elif caltype == 2:
                        fp.write('{0:.4f}'.format(vcal[caltype][ch][i]))
                    else:
                        if tcal is not None:
                            fp.write('{0:.4f}'.format(tcal[ch][i]))
                        else:
                            fp.write('0.5')
                    if i < tracelen-1:
                        fp.write(' ')
                fp.write('\n')
    print(f'Saved calibration to {txtfile}')
    
    if npy:
        cal = np.empty((4,nchan,tracelen))
        cal[:3] = vcal
        if tcal is not None:
            cal[3] = tcal
        else:
            cal[3] = np.zeros(cal[3].shape) + 0.5
        npyfile = txtfile[:-4]+'.npy'
        np.save(npyfile,cal)
        print(f'Saved calibration to {npyfile}')
    return
    
def loadCalibration(calfile):
    # handle txt
    if calfile[-4:] == '.txt':
        cal_raw = np.loadtxt(calfile)
        nchan = int(cal_raw.shape[0]/4)
        tracelen = cal_raw.shape[1]
        cal = np.reshape(cal_raw,(nchan,4,tracelen))
        cal = np.swapaxes(cal,0,1)
    else: # npy
        cal = np.load(calfile)
    return cal

def voltageCalibration(gbf,gbf2=None,dv=0):
    '''voltageCalibration: amplitude calibration required by DRS4

    From Stricker-Shaver et al. 2014:
     "Firstly, the voltages of the stored waveform show slightly different 
      offsets and gains for each sampling cell [...]
      Secondly, a time-dependent readout offset correction is performed [...]
      Thirdly, the gain correction for each cell is done"

    Parameters
    ----------
    gbf: vcal gbf (or .dat file name)
    gbf2: second vcal gbf (or .dat file name)
    dv: voltage difference gbf2 - gbf, in mV
    '''
    # check that all traces are unpacked
    if type(gbf) is str:
        gbf = tofpy.load(gbf)
    elif gbf.traces is None:
        gbf.unpackTraces()
    trcopy = gbf.traces.copy()
    nchan,tracelen = trcopy[0].shape
    vcal = np.ones((3,nchan,tracelen))

    print('Voltage calibration...')
    # make numpy array with traces rolled so that index0 = cell0
    rolled_traces = np.zeros(trcopy.shape)
    trcopy[:,:,:gbf.nskip] = np.nan # ignore first n cells for vcal1
    for i in range(len(trcopy)):
        rolled_traces[i] = np.roll(trcopy[i],gbf.tcells[i],axis=1)
    
    # vcal1
    vcal[0] = np.nanmedian(rolled_traces,axis=0)
    # vcal2
    for i in range(len(trcopy)): # have to roll vcal every event
        trcopy[i] -= np.roll(vcal[0],-gbf.tcells[i],axis=1)
    #print(np.nanmedian(trcopy, axis=0).shape)
    vcal[1] = np.nanmedian(trcopy,axis=0) # median of all traces
    vcal[1][:,:gbf.nskip] = 0 # ignore first n cells
    # vcal3
    if gbf2 is not None:
        # WARNING: R E C U R S I O N
        gainvcal = voltageCalibration(gbf2)
        vcal[2] = dv/(gainvcal[0] - vcal[0]) # in mV/ADC
    return vcal
    
def applyVCal(gbf,vcal,clean=True):
    # overwrite gbf traces with voltage-calibrated ones
    if gbf.vcaldone == True:
        print('WARNING: Already applied voltage cal.')
        return
    gaincorr = True
    if np.any(vcal[2]==1): # no cal
        gaincorr = False
        print('WARNING: No gain correction')
    if clean:
        print('Spike cleaning enabled')
    if gbf.traces is None:
        gbf.unpackTraces()
    nevents,nchan,tracelen = gbf.traces.shape
    for i in range(nevents):
        trace_cal = np.roll(vcal[0],-gbf.tcells[i],axis=1)
        gbf.traces[i] -= trace_cal
        # FIXME - disabled this for debugging
        gbf.traces[i] -= vcal[1]
        gbf.traces[i] *= vcal[2]
        clean = False
        if clean:
            gbf.traces[i] = cleanSpikes(gbf.traces[i],gaincorr)
    gbf.vcaldone = True
    return
    

def timingCalibration(gbf,vcal=None,local=False,calfreq=0.025,edge='average'):
    '''voltageCalibration: amplitude calibration required by DRS4

    From Stricker-Shaver et al. 2014:
     "The first part estimates the effective sampling intervals by measuring
      voltage differences between two neighboring cells and is called 'local'
      The second part refines the sampling intervals by measuring time diff-
      erences between cells that are far apart and is therefore called 'global'"

    Parameters
    ----------
    vcal: voltage calibration array
    local: only do local tcal
    calfreq: calibration sine wave frequency in GHz. default is 25 MHz
    edge: part of sine to use - 'falling', 'rising', or 'average'.
    '''
    # check that all traces are unpacked
    if type(gbf) is str:
        gbf = tofpy.load(gbf)
    elif gbf.traces is None:
        gbf.unpackTraces()
    # check vcal
    print(gbf.trignums[:10])
    print(gbf.trignums[12], 'debbuging event')
    print (len(gbf.trignums), 'n events')
    if gbf.vcaldone == False:
        if vcal is not None:
            applyVCal(gbf,vcal)
        else:
            print('WARNING: Voltage cal. must be done before timing cal.')
            return gbf.cellwidths
    tcal = np.ones(gbf.cellwidths.shape) / gbf.nominalFreq
    
    # Local (following DRSBoard::AnalyzeSlope)
    trcopy = gbf.traces.copy()
    #print (f' first 20 vals {trcopy[12][0][:20]}')
    trcopy[:,:,:gbf.nskip] = np.nan # ignore first n cells
    sinmax = 60 # ~1000 ADC units
    dvcut = 15 # ns away that should be considered
    trcopy[np.abs(trcopy) > sinmax] = np.nan # only use ~linear part of sine wave
    nnans = 0
    #raise 
    # rotate traces + get avg dv
    rolled_traces = np.zeros(trcopy.shape)
    drolled_traces= np.zeros(trcopy.shape) # diff
    for i in range(len(trcopy)):
        rolled_traces[i] = np.roll(trcopy[i],gbf.tcells[i],axis=1)
    drolled_traces[:,:,:-1]=rolled_traces[:,:,1:]-rolled_traces[:,:,:-1]
    drolled_traces[:,:,-1] =rolled_traces[:,:,0] - rolled_traces[:,:,-1]
    if edge[:3] == 'ris' or edge[:2] == 'av':
        drolled_traces[drolled_traces<0] = np.nan
    elif edge[:3] == 'fal':
         drolled_traces[drolled_traces>0] = np.nan
    drolled_traces = np.abs(drolled_traces)
    # should be ~15
    drolled_traces[np.abs(drolled_traces-15)>dvcut]=np.nan
    nnans = 0
    for k in range(1024):
        if np.isnan(drolled_traces[0][0][k]):
            nnans += 1
        print (k, drolled_traces[0][0][k])
    print(f'We saw {nnans} nans')
    dvs = np.nanmean(drolled_traces,axis=0) # ch,nev
    print (dvs.shape)
    for k in range(len(dvs)):
        print (k, dvs[k])
    print ('------- -------')
    for k in range(1024):
        print (k, dvs[0][k])
    raise
    if np.any(np.isnan(dvs)):
        print('WARNING: NaNs in cellwidths')
        dvs[np.isnan(dvs)] = np.nanmean(dvs)
        
    tcal *= dvs / np.mean(dvs,axis=1).reshape((len(tcal),1))
    if edge[:2] == 'av':
        tcalfall = timingCalibration(gbf,vcal=vcal,local=True,
                                     calfreq=calfreq,edge='falling')
        tcal = (tcal+tcalfall)/2.0 # average
        edge='rising' # for global
  
    for k in range(10):
        print(f"tcal avl {tcal[0][k]}")
    if local:
        return tcal
    #return tcal 
    # Global (following DRSBoard::AnalyzePeriod)
    nevents,nchan,tracelen = gbf.traces.shape
    damping = 0.1
    corr_limit = 0.05
    nIterPeriod = 1000 #500 or nevents #
    #nIterSlope  = 500
    n_correct = np.zeros(nchan,dtype=int)
    nperiod = gbf.nominalFreq/calfreq
    for i in range(nIterPeriod):
        tcell = gbf.tcells[i]
        this_event_id = gbf.trignums[i]
        # rolled time bin widths
        print('this_event_id', this_event_id)
        if this_event_id != 6374206:
            continue
        print('nperiod', nperiod)
        print(tcell)    
        print ('rolling...')
        dts = np.roll(tcal,-tcell,axis=1)
        print (tcal)
        print (dts)
        raise
        for k in range(10):
            print(dts[0][k])
        for ch in range(nchan):         
            for k in range(10):
                print(gbf.traces[i][ch][k])
            #raise
            zcs,periods = getPeriods(gbf.traces[i,ch],dts[ch],nperiod,gbf.nskip,edge)
            print(f"==> Will iterate over {len(periods)} periods")
            for j in range(len(periods)):
                period = periods[j]
                zca = zcs[j] + tcell   # index of first zc
                zcb = zcs[j+1] + tcell # index of second zc
                # apply correction
                corr = (1.0/calfreq)/period
                if edge == '':
                    corr *= 0.5 # using half periods
                if abs(corr-1) > corr_limit:
                    continue
                #damping = gbf.nominalFrequency / nIter * 2
                print ('tcell',tcell)
                print ('period', period)
                print ('event_id', this_event_id)
                corr = (corr-1)*damping + 1
                print ('corr',corr)
                print (zca, zcb, tracelen)
                #if this_event_id != 6374206:
                #    continue
                #corr = 1.0
                #raise
                if zca < tracelen and zcb > tracelen:
                    tcal[ch][zca:] *= corr
                    tcal[ch][:zcb%tracelen] *= corr
                    print("ERROR")
                    #raise
                else:
                    tcal[ch][zca%tracelen:zcb%tracelen] *= corr
                # applied!
                n_correct[ch] += 1
        damping *= 0.99
    return tcal

def applyTCal(gbf,tcal):
    if gbf.tcaldone == True:
        print('WARNING: Already applied timing cal.')
    gbf.cellwidths = tcal
    gbf.tcaldone = True
    return
    
def applyCal(gbf,cal):
    applyVCal(cal[:3])
    applyTCal(cal[3])

def getPeriods(trace,dts,nperiod,nskip=0,edge=''):
    # subtract pedestal/offset
    periods  = []
    firstbin = 20
    lastbin  = firstbin + int(nperiod * int(900/nperiod))
    # TODO: mean is better, but fails is sine is truncated
    print (firstbin, lastbin)
    print (np.median(trace[firstbin:lastbin]))
    print (trace[firstbin:lastbin])
    print ('---')
    trace   -= np.median(trace[firstbin:lastbin])
    
    #zero crossings are indices BEFORE cross [[ev,ch,zc],...]
    zcs      = np.where(np.diff(np.signbit(trace)))[0]
    zcs      = zcs[zcs>nskip]
    # choose +-
    if edge[:3] == 'ris':
        zcs = [zc for zc in zcs if trace[zc] < 0]
    elif edge[:3] == 'fal':
        zcs = [zc for zc in zcs if trace[zc] > 0]
    if len(zcs) < 3:
        return zcs,periods
    print(zcs)
    print(len(zcs), 'zero crossings')
    for i in range(len(zcs)-1):
        zca = zcs[i]   # index of first zc
        zcb = zcs[i+1] # index of second zc
        print (zca+1, zcb)
        n_zeros = 0
        for baz in dts[zca+1:zcb]:
            if baz == 0:
                n_zeros += 1
            else:
                print(f'dts : {baz}')
        print (f"We got {n_zeros} zero elements in dts!")
        period = np.sum(dts[zca+1:zcb]) # doesn't include bin fractions
       
        print("period step 1", period)
        #raise
        tra = trace[zca:zca+2]
        period += dts[zca]*abs(tra[1]/(tra[1]-tra[0])) # first semi-bin
        print("period step 2", period)
        trb = trace[zcb:zcb+2]
        period += dts[zcb]*abs(trb[0]/(trb[1]-trb[0])) # last semi-bin
        print("period step 3", period)
        # check for anomalies
        if np.abs((zcb-zca)-nperiod) > 5:
            zcs = zcs[:i+1]
            break
        print(f'getPeriods zca, zcb {zca} {zcb} {period}');
        periods.append(period)
    return zcs,periods
