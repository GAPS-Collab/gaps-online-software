# Licensed under a 3-clause BSD style license - see PYFITS.rst

import sys

import numpy as np
import matplotlib.pyplot as plt

from .calibration import getPeriods, loadCalibration
from .parsing import cleanSpikes

__all__ = ['tracePlotter','plotEvent','plotCal','TPtest']

def tracePlotter(gbf,event=0,calfile=None,clean=True,prms=False,tns=False):
	'''tracePlotter: basic plotting program that can use calibrations

	Parameters
	----------
	calfile: calibration file
	clean: spike cleaning
	prms: print RMS
	tns: plot time in ns, rather than cell number
	'''
	# instructions
	print('Instructions:')
	print(' `enter` goes to next event')
	print(' `# + enter` goes to event number #')
	print(' `q + enter` quits')
	# set up plots
	fig = plt.figure()
	axes = []
	for i in range(9):
		axes.append(fig.add_subplot(3,3,i+1))
	# load calibrations
	vcal = None
	tcal = None
	if calfile is not None:
		cal = loadCalibration(calfile)
		vcal = cal[:3]
		if np.any(cal[3] != 1./gbf.nominalFreq):
			tcal = cal[3]
	# plot loop
	eventnum = event
	while True:
		# use unpacked traces, if available
		if gbf.traces is not None:
			traces = gbf.traces[eventnum]
			tcell = gbf.tcells[eventnum]
			trignum = gbf.trignums[eventnum]
			cell0 = 1024-tcell
		else:
			# parse up to `event`
			if gbf.nparsed < eventnum + 1:
				gbf.parse(eventnum)
			header,tcell,traces = gbf._unpack(gbf.packets[eventnum])
			trignum = header['event_cnt']
			cell0 = 1024-tcell
		nchan,tracelen = traces.shape
		# calibrate
		if vcal is not None and not gbf.vcaldone:
			traces -= np.roll(vcal[0],-tcell,axis=1)
			traces -= vcal[1]
			traces *= vcal[2]
		if tns:
			if tcal is not None and not gbf.tcaldone:
				ts = gbf.timesFromCellWidths(tcell,tcal)
			else:
				t = np.arange(len(traces[0])) * 1./gbf.nominalFreq
				ts = np.tile(t,(nchan,1))
		else:
			t = np.arange(len(traces[0]))
			ts = np.tile(t,(nchan,1))
		# spikes
		if clean:
			if vcal is not None or gbf.vcaldone:
				traces = cleanSpikes(traces,True)
			else:
				traces = cleanSpikes(traces)
		# plot
		print(f'Event # {eventnum}   (Trigger # {trignum})')
		for ch in range(nchan):
			if prms:
				rms = np.std(traces[ch][20:])
				print(ch, rms, rms/np.nanmedian(vcal[2][ch]))
			axes[ch].lines = [] # remove old trace
			axes[ch].plot(ts[i],traces[ch],color='k')
			if tns:
				axes[ch].axvline(ts[ch,cell0],color='k',alpha=.3,ls='--')
			else:
				axes[ch].axvline(cell0,color='k',alpha=.3,ls='--')
		axes[1].set_title(f'event # {eventnum}')   
		fig.canvas.draw()
		fig.canvas.flush_events()

		next_eventnum = input()
		if next_eventnum == 'q':
			sys.exit()
		try:
			eventnum = int(next_eventnum)
		except:
			eventnum += 1
			
def plotEvent(gbf,eventnum=0,ch=None,ax=None):
	'''plotEvent: plot single event

	Parameters
	----------
	vcfile: voltage calibration file
	tcfile: timing calibration file
	'''
	# use unpacked traces, if available
	if gbf.traces is not None:
		traces = gbf.traces[eventnum]
		tcell = gbf.tcells[eventnum]
	else:
		# parse up to `event`
		if gbf.nparsed < eventnum + 1:
			gbf.parse(eventnum)
		header,tcell,traces = gbf._unpack(gbf.packets[eventnum])
	# get nchan
	nchan,tracelen = traces.shape
	# plot
	if ax is not None and ch is not None:
		ax.plot(traces[ch],color='k')
	else:
		fig = plt.figure()
		axes = []
		for ch in range(nchan):
			axes.append(fig.add_subplot(3,3,i+1))
			axes[ch].plot(traces[ch],color='k')
		axes[1].set_title(f'event # {eventnum}')

def plotCal(calfile,flush=False):
	'''plotCal: look at voltage or timing calibrations'''
	cal = loadCalibration(calfile)
	nfigs,naxes,tracelen = cal.shape # 4, nchan, nsamp
	for i in range(nfigs):
		fig=plt.figure(i)
		for j in range(naxes):
			plt.subplot(3,3,1+j)
			plt.plot(cal[i][j],'k-')
		if flush:
			fig.canvas.draw()
			fig.canvas.flush_events()
	
def TPtest(gbf,nevents=None,ch=None,color=None,ax=None,bins=None,edge=''):
	'''TPtest: time period test from Stricker-Shaver 2014'''
	# RB v2.4s have 250 mVpp 25 MHz sine ref
	print('Time Period Test')
	ntot,nchan,tracelen = gbf.traces.shape
	if nevents is None:
		nevents = ntot
	chs = range(nchan)
	if ch is not None:
		if type(ch) is not int:
			print('ERROR: channel must be an integer')
			return [0]
		chs=[ch]
	c = color # for plotting
	
	# getPeriods vars
	nskip = gbf.nskip
	nperiod = gbf.nominalFreq/25e-3 # number of bins in 25 MHz period
	if bins is None:
		tperiod = nperiod/gbf.nominalFreq
		if edge == '':
			tperiod /= 2.0
		bins=np.arange(tperiod-5,tperiod+5,.1)
	
	tps = [[] for ch in range(nchan)] # up to 8 empty arrays
	for i in range(nevents):
		# rolled time bin widths
		dts = np.roll(gbf.cellwidths,-gbf.tcells[i],axis=1)
		if gbf.verbose >= 1 and i % 10 == 0:
			print(i)
		for ch in chs:
			zcs,periods = getPeriods(gbf.traces[i,ch],dts[ch],nperiod,nskip,edge)
			tps[ch] += periods
	for ch in chs:
		pmed= np.median(tps[ch])
		plo = pmed-np.percentile(tps[ch],50-68.3/2)
		phi = np.percentile(tps[ch],50+68.3/2)-pmed
		print('ch{0}: {1:.3f} + {2:.3f} - {3:.3f}'.format(ch,pmed,phi,plo))
		if color is None:
			c=f'C{ch}'
		if ax is not None:
			ax.hist(tps[ch],bins=bins,histtype='step',color=c)
			#ax.hist(tps[ch],bins=np.arange(15,25,.1),color=c,alpha=0.4)
		else:
			plt.hist(tps[ch],bins=bins,histtype='step',color=c)
			#plt.hist(tps[ch],bins=np.arange(15,25,.1),color=c,alpha=0.4)
	return [np.std(x) for x in tps]


