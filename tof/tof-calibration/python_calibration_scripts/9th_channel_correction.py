'''
Read in blob2root output, find pulse times, measure ch9 phases, plot
'''

import sys
import numpy as np
import matplotlib.pyplot as plt
import ROOT
from scipy.optimize import minimize

# definitions
datadir = '/Users/jamieryan/data/gaps_data/ch9nov5/'
fn = datadir+'timingtest3.root'
fn_ev = datadir+'event_list.txt'
N = 1024 # number of cells in trace
nchan = 18 # number of channels
pedwindow=[20,700] # pedestal window (cell #s)
pulsewindow=[750,850] # pulse window (cell #s)
pulse_chs = (0,9)
sine_chs = (8,17)

# load eventlist
eventlist = np.loadtxt(fn_ev,dtype=int)

# load traces
print('Loading...')
f = ROOT.TFile.Open(fn, 'read')
rec = f.rec
nentries = rec.GetEntriesFast()
t = np.empty((nentries,nchan,N))	# time
chn = np.empty((nentries,nchan,N))	# data
tcells = np.empty(nentries)
for i in range(nentries):
	if i % 100 == 0:
		print(i, '\r', end='')
		sys.stdout.flush()
	rec.GetEntry(i)	# changes entry accessed by default
	for c in range(nchan):
		t[i][c] = np.array( getattr(rec,'t'+str(c)) )
		chn[i][c] = np.array( getattr(rec,'chn'+str(c)) )
f.Close()

# spike removal
def cleanSpikes(traces):
	thresh = 16
	spikefilter = -traces[:,:-3]+traces[:,1:-2]+traces[:,2:-1]-traces[:,3:]
	spikes = np.where(np.sum(spikefilter > thresh,axis=0) >= 2)[0]
	for i in spikes:
		dV = (traces[:,i+3]-traces[:,i])/3.0
		traces[:,i+1] = traces[:,i] + dV
		traces[:,i+2] = traces[:,i] + 2*dV
	return traces
for rb in range(int(len(chn)/9)):
	for i in range(len(chn)):
		chn[i,9*rb:9*(rb+1)] = cleanSpikes(chn[i,9*rb:9*(rb+1)])

# pedestal subtraction
peds = np.mean(chn[:,:,pedwindow[0]:pedwindow[1]],axis=2)
t0,t1 = t[:,pulse_chs[0]],t[:,pulse_chs[1]]
ped0 = np.mean(chn[:,pulse_chs[0],pedwindow[0]:pedwindow[1]],axis=1)
ped1 = np.mean(chn[:,pulse_chs[1],pedwindow[0]:pedwindow[1]],axis=1)
v0 = chn[:,pulse_chs[0]] - np.reshape(ped0,(len(t),1))
v1 = chn[:,pulse_chs[1]] - np.reshape(ped1,(len(t),1))

# bootleg CFD
def calcTDCs(t,chn,cfd_frac=0.2,thresh=5,pulsewindow=pulsewindow):
	# t: time array
	# chn: voltage array
	# cfd_frac: CFD fraction
	# thresh: minimum pulse height (to omit no-pulse events)
	nentries = len(t)
	tdcs = np.zeros(nentries)
	for i in range(nentries):
		ci = chn[i]
		ti = t[i]
		# check for threshold crossing
		if np.count_nonzero(ci > thresh) == 0:
			tdcs[i] = np.nan
			continue
		# find threshold
		ipeak = pulsewindow[0]+np.argmax(ci[pulsewindow[0]:pulsewindow[1]])
		y = ci[ipeak] * cfd_frac
		# find bin before cross
		icross = ipeak - np.argmax(ci[:ipeak][::-1]<y) - 1
		# interp
		slope = (ti[icross+1]-ti[icross])/(ci[icross+1]-ci[icross])
		t_interp = (y - ci[icross])*slope + ti[icross]
		tdcs[i] = t_interp
	return tdcs
tdc0 = calcTDCs(t0,v0)
tdc1 = calcTDCs(t1,v1)
tdif = tdc0-tdc1

# fit ch9 sine waves
freq = 0.021486 # ~ 25 MHz...
angfreq = 2*np.pi*freq # 0.1350
def logistic(x): # maps +/- inf to +/- 1; helps with fitting
	return 2/(1+np.exp(-x)) - 1
def sine(x,p): # amp freq phase offset
	omega = angfreq + 0.1*logistic(p[1]) # don't let this reach 2x omega
	return p[0]*np.sin(x*omega+p[2])+p[3]
sines = chn[:,sine_chs]
sints = t[:,sine_chs]
sine_pars = np.zeros((len(t),2,4))
for i in range(len(sines)):
	for b in range(2): # `b` for board!
		s = sines[i,b]
		st= sints[i,b]
		# find 0
		mean_guess = np.median(s[20:951]) # ~10 periods
		#mean_guess = np.mean(np.sort(s[20:])[400:-400])
		phase_guess = -2*np.pi*((st[5+np.argmax(s[5:100])]*freq)%1 - 0.25)
		x0 = (261,angfreq,phase_guess,mean_guess)
		# fit V > -90 mV data
		s90 = s[s>-90]
		st90 = st[s>-90]
		minfunc = lambda p: np.sum((s90-sine(st90,p))**2) # least squares
		res = minimize(minfunc,x0=x0,method='Powell')
		sine_pars[i,b] = res.x

# find phase difference
# - calc phase shifts forward + backward, then choose smaller one
# - phase defined as amount 0th channel lags 1st channel (ns)
phasedif = np.zeros(len(sines)) # in nanoseconds
for i in range(len(sines)):
	phaseA = sine_pars[i,0,2] % (2*np.pi)
	phaseB = sine_pars[i,1,2] % (2*np.pi)
	candidates = [(phaseA-phaseB) % (2*np.pi), (phaseB-phaseA) % (2*np.pi)]
	ind = np.argmin(candidates)
	phasedif[i] = candidates[ind] / angfreq # convert to ns
	if ind == 0:
		phasedif[i] *= -1


# Plots	
		
# CFD TDC plot
plt.figure(figsize=(6,3))
bins=np.arange(-5,5,.1)
plt.hist(tdif,bins=bins,label='All')
plt.hist(tdif[eventlist],bins=bins,label='Event list')
plt.xlabel('CFD time difference (ns)')
plt.ylabel('Number of events')
plt.legend()
plt.tight_layout()
plt.text(-4.5,45,'All RMS = {0:.2f} ns'.format(np.nanstd(tdif)))
plt.text(-4.5,40,'RMS = {0:.2f} ns'.format(np.nanstd(tdif[eventlist])))
plt.text(-4.5,35,r'$|\Delta t|<2$ ns RMS = 0.57 ns')


# ch9 sine fit example
plt.figure(figsize=(6,3))
i = 0
x=np.arange(1024)*.5
ax=plt.subplot(211)
plt.title('Event {0}'.format(eventlist[0]))
plt.plot(sints[i,0],sines[i,0],'k-')
plt.plot(x,sine(x,sine_pars[i,0]),'r--')
plt.xlim(0,512)
plt.xticks([])
plt.ylabel('Voltage (mV)')
plt.subplot(212,sharey=ax)
plt.plot(sints[i,1],sines[i,1],'k-')
plt.plot(x,sine(x,sine_pars[i,1]),'r--')
plt.xlim(0,512)
plt.xlabel('Time (ns)')
plt.ylabel('Voltage (mV)')
plt.tight_layout()


# phase difference vs. event number
plt.figure(figsize=(6,3))
n=np.arange(len(phasedif))
plt.plot(n,phasedif,'.',color='c',alpha=1)
plt.plot(n[eventlist],phasedif[eventlist],'k.')
plt.xlabel('Event number')
plt.ylabel('Phase diff (ns)')
plt.tight_layout()


# exploratory plotting
plt.figure(figsize=(6,5))
plt.scatter(phasedif[eventlist],tdif[eventlist],c=n[eventlist],cmap='jet')
#plt.plot(tdif[eventlist],c=phasedif[eventlist]
#ax=plt.subplot(211)
#plt.plot(n[eventlist],tdif[eventlist],'k.')
#plt.subplot(212,sharex=ax)
#plt.plot(n[eventlist],phasedif[eventlist],'k.')
#plt.plot(n[eventlist],np.diff(phasedif)[eventlist-1],'k.')
#plt.scatter(n[eventlist],tdif[eventlist],c=phasedif[eventlist],cmap='jet')
#plt.scatter(n[eventlist],tdif[eventlist],c=np.diff(phasedif)[eventlist-1],cmap='jet')
plt.colorbar().set_label('Event number')
plt.xlabel('Phase diff (ns)')
plt.ylabel('CFD time difference (ns)')
plt.tight_layout()


