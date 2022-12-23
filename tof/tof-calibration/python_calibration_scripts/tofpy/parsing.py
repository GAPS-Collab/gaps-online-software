# Licensed under a 3-clause BSD style license - see PYFITS.rst

import struct

import numpy as np
import libscrc

__all__ = ['GBF', 'load']

class GBF(object):
	''' GAPS Bank File '''

	def __init__(self,infile=None):
		# Values fixed in firmware
		self.head = b'\xaa\xaa'
		self.tail = b'\x55\x55'
		self.header_bytes = {
			'head': 2,
			'status': 2,
			'len': 2,
			'roi': 2,
			'dna': 8,
			'fw_hash': 2,
			'id': 2,
			'ch_mask': 2,
			'event_cnt': 4,
			'dtap0': 2,
			'dtap1': 2,
			'timestamp': 6
		}
		self.trace_bytes = {
			'head': 2,
			'crc32': 4
		}
		self.footer_bytes = {
			'stop_cell': 2,
			'crc32': 4,
			'tail': 2
		}
		self.nominalFreq = 2 # GHz
		self.nskip = 2 # calibrations skip first n cells after trigger cells
		#
		self.index = 2 # this implementation of unpacking starts at index 2...
		self.nbad = 0
		self.data = None
		self.verbose = 1

		self.packets = []
		self.nparsed = 0
		self.all_parsed = False # packets parsed (not necessarily unpacked)

		self.traces = None # dims: (packets, channels, ADC values)
		self.tcells = None # trigger cell numbers
		self.trignums = None # trigger number
		self.timestamps = None # 48-bit number of clock cycles
		self.cellwidths = None # time-widths of cells, in ns
		self.dna = None # board DNA
		#self.nunpacked = 0
		#self.all_unpacked = False
		self.vcaldone = False
		self.tcaldone = False

		# load binary data from file
		if infile is not None:
			if self.verbose >= 1:
				print(f'Loading {infile}')
			with open(infile, 'rb') as fp:
				self.data = fp.read()

	def parse(self,event=None):
		'''parse: find packets within data and store

		Parameters
		----------
		event: if not `None`, parse only up to packet # `event`
		'''
		if self.data is None:
			print('No data to parse!')
			return
		if self.all_parsed:
			print('All packets parsed!')
			return
		if event is not None:
			if self.nparsed > event: # nothing to do
				return

		# count number of bytes in header before packet len; = 4
		len_dict_ind = list(self.header_bytes.keys()).index('len')
		len_ind = np.sum(list(self.header_bytes.values())[:len_dict_ind])

		if self.verbose >= 1:
			print('Entering packet loop')
		while True:
			if self.index > len(self.data)-2: # end of file
				self.all_parsed = True
				break
			if self.data[self.index-2:self.index] != self.head:
				self.index += 1
				continue
			# found packet head! slice out packet
			s = self.index - 2 # start index
			len_bytes = self.data[s+len_ind:s+len_ind+self.header_bytes['len']]
			pkt_len = struct.unpack('<H', len_bytes)[0]
			pkt_len *= 2 # convert words to bytes
			packet = self.data[s:s+pkt_len]
			# check for tail
			if packet[-2:] != self.tail:
				self.index += 1
				self.nbad += 1
				continue
			# good packet (possibly)! append
			self.packets.append(packet)
			self.nparsed += 1
			self.index += pkt_len # move index past good packet

			if event is not None:
				if self.nparsed > event:
					return

		if self.verbose >= 1:
			print('Exited packet loop')

	def unpackTraces(self):
		'''unpackTraces: put all traces in numpy arrays'''

		if self.nparsed < 1:
			print('No traces to unpack')
			return

		# get nchan
		# TODO: may be different nchan in general, eventually?
		header,tcell,traces = self._unpack(self.packets[0])
		nchan,tracelen = traces.shape
		self.traces = np.empty((self.nparsed,nchan,tracelen))
		self.tcells = np.zeros(self.nparsed,dtype=int)
		self.trignums = np.zeros(self.nparsed,dtype=int)
		self.timestamps = np.zeros(self.nparsed,dtype=int)
		self.cellwidths = np.zeros((nchan,tracelen)) + 1.0/self.nominalFreq
		self.traces[0] = traces
		self.tcells[0] = tcell
		self.trignums[0] = header['event_cnt']
		self.timestamps[0] = header['timestamp']
		self.dna = header['dna']
		#trignum = header['event_cnt']

		if self.verbose >= 1:
			print('Unpacking traces...')
		for i in range(1,self.nparsed):
			if self.verbose >= 1 and i % 500 == 0:
				print(i)
			header,tcell,traces = self._unpack(self.packets[i])
			try:
				self.traces[i] = traces
				self.tcells[i] = tcell
				self.trignums[i] = header['event_cnt']
				self.timestamps[i] = header['timestamp']
			except:
				print('WARNING: Error unpacking evt {0}, using prev evts only'.format(i))
				self.traces = self.traces[:i]
				self.tcells = self.tcells[:i]
				self.trignums = self.trignums[:i]
				self.timestamps = self.timestamps[:i]
				break
			
	def timesFromCellWidths(self,tcell,tcal=None):
		'''timesFromCellWidths: build time array from widths and trigger cell
		
		Parameters
		----------
		tcell: trigger cell
		tcal: optionally, use times other than self.cellwidths
		'''
		if tcal is not None:
			cws = tcal
		else:
			if self.cellwidths is None:
				print('ERROR: No cell widths')
				return
			cws = self.cellwidths
		times = np.zeros(cws.shape)
		# t0 = 0, ti = dt0 + ... + dti-1
		for i in range(1,times.shape[1]):
			times[:,i] = times[:,i-1] + cws[:,(tcell+i-1)%1024]
		return times
		
	def rbFromDNA(self):
		''' return RB number, using unique Zynq DNA '''
		dnas = {77380906573213780: 1,
				9609908518406236:  2,
				78985381379328092: 3,
				24942185850882132: 4,
				25003068095826012: 5,
				32831594097035348: 6}
		return dnas[self.dna]


	# unpack 32-bit word in header
	def _unpack_32(self,x):
		ba = bytearray(x)
		# Perform word swap
		ba[:]=ba[1],ba[0],ba[3],ba[2]
		return int.from_bytes(bytes(ba), 'big')
		
	# unpack 48-bit word in header
	def _unpack_48(self,x):
		ba = bytearray(x)
		# Perform word swap
		ba[:]=ba[1],ba[0],ba[3],ba[2],ba[5],ba[4]
		return int.from_bytes(bytes(ba), 'big')
		
	# unpack 64-bit word in header
	def _unpack_64(self,x):
		ba = bytearray(x)
		# Perform word swap
		ba[:]=ba[1],ba[0],ba[3],ba[2],ba[5],ba[4],ba[7],ba[6]
		return int.from_bytes(bytes(ba), 'big')

	# unpack packet
	def _unpack(self,pkt):
		index = 0
		header = {}
		for field in self.header_bytes.keys():
			next_index = index + self.header_bytes[field]
			if field == 'status':
				status = struct.unpack('<H', pkt[index:next_index])[0]
				status_str = '{:016b}'.format(status) # binary
				sync = status_str[-1]
				drs_busy = status_str[-2]
				zynq_temp = int(status_str[:12], 2) # units TBD
				if sync != '0':
					print('WARNING: sync error')
				if drs_busy != '0':
					print('WARNING: DRS4 was busy (lost trigger)')
			elif field == 'roi': # gives trace length (cells) - 1
				header[field] = struct.unpack('<H', pkt[index:next_index])[0]
			elif field == 'ch_mask':
				mask = struct.unpack('<H', pkt[index:next_index])[0]
				mask_str = '{:016b}'.format(mask)[7:] # binary
				header[field] = mask_str
				# number of channels uses first 9 bits ONLY
				# 10th bit is related to AUTO_9TH_CHANNEL
				nchan = mask_str.count('1')
			elif field == 'event_cnt':
				header[field] = self._unpack_32(pkt[index:next_index])
			elif field == 'timestamp':
				header[field] = self._unpack_48(pkt[index:next_index])
			elif field == 'dna':
				header[field] = self._unpack_64(pkt[index:next_index])
			else:
				# not handled yet
				header[field] = pkt[index:next_index]
			index = next_index
		# get stop cell (int) a.k.a. trigger cell
		stop_index = -self.footer_bytes['tail'] - self.footer_bytes['crc32'] - self.footer_bytes['stop_cell']
		stopcell = struct.unpack('<H', pkt[stop_index:stop_index+self.footer_bytes['stop_cell']])[0]
		# cyclic redundancy check for packet
		# JLR TODO: check this is actually what SPQ firmware does
		crc_index = -self.footer_bytes['tail'] - self.footer_bytes['crc32']
		packet_crc32 = self._unpack_32(pkt[crc_index:crc_index+self.footer_bytes['crc32']])
		packet_crc32_calc = libscrc.crc32(pkt[:crc_index])
		if packet_crc32_calc != packet_crc32: # print if mismatch
			print('WARNING: packet CRC32 mismatch')
		# set up trace + cell # arrays
		tracelen = header['roi'] + 1
		tracelen_bytes = 2 * tracelen # convert to bytes
		#cellnums = np.roll(np.arange(tracelen), -stopcell)
		traces = np.zeros((nchan,tracelen))
		# now read traces from "payload"
		# `adc` extracts ADC val from 2-byte word
		#   each value is 14 bits ADC data, followed by 2 bits "parity"
		adc = lambda x: int(bin(x)[2:].rjust(16,'0')[2:],2)
		for i in range(nchan):
			# get channel ID
			ch = struct.unpack('<H', pkt[index:index+self.trace_bytes['head']])[0]
			index += self.trace_bytes['head']
			# get ADC vals
			traceblob = pkt[index:index+tracelen_bytes]
			traceblob_ints = struct.unpack(f'<{tracelen}H', traceblob)
			trace = list(map(adc, traceblob_ints))
			traces[i] = np.array(trace)
			index += tracelen_bytes
			# cyclic redundancy check for channel
			channel_crc32 = self._unpack_32(pkt[index:index+self.trace_bytes['crc32']])
			channel_crc32_calc = libscrc.crc32(traceblob)
			if channel_crc32_calc != channel_crc32: # print if mismatch
				print(f'WARNING: channel {i} CRC32 mismatch')
			# and move to next trace
			index += self.trace_bytes['crc32']
		return header, stopcell, traces


def load(infile,unpackall=True):
	g = GBF(infile)
	g.parse()
	if unpackall:
		g.unpackTraces()
	return g
	
def cleanSpikes(traces,vcaldone=False):
	# TODO: make robust (symmetric, doubles, fixed/estimated spike height)
	thresh = 360
	if vcaldone:
		thresh = 16
	spikefilter = -traces[:,:-3]+traces[:,1:-2]+traces[:,2:-1]-traces[:,3:]
	spikes = np.where(np.sum(spikefilter > thresh,axis=0) >= 2)[0]
	for i in spikes:
		dV = (traces[:,i+3]-traces[:,i])/3.0
		traces[:,i+1] = traces[:,i] + dV
		traces[:,i+2] = traces[:,i] + 2*dV
	return traces

