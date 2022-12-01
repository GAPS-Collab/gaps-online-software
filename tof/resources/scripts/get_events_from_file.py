#! /usr/bin/env python

import _pyTofServer as ptf

import tqdm

import hepbasestack as hep
hep.visual.set_style_present()
import dashi as d
import pylab as p
d.visual()

from rich.progress import Progress
import time

BLOBEVTSIZE = ptf.get_current_blobevent_size()

print (BLOBEVTSIZE)

class BitSet:

    def __init__(self, bitset):
        if hasattr(bitset, "__iter__"):
            self.bitset = bitset
        else:
            self.bitset = [bitset]

    def __len__(self):
        return len(self.bitset)

    def empty(self):
        return 0
    
    def set_bit(self,i):
        self.bitset |= 1 << i

    def read_bit(self,i):
        return self.bitset &1 << i

    def reset_bit(self,i):
        self.bitset &= ~(1 << i)

    def flip_bit(self,i):
        self.bitset ^= 1 << i

    def flip_all(self):
        return ~self.bitset 

    def to_number(self):
        raise
        output = ''
        for i,k in enumerate(self.bitset):
            if i >0:
                try:
                    output += bin(k)[2:]
                except:
                    output += bin(ord(k))[2:]
            else:
                try:
                    output += bin(k)
                except:
                    output += bin(ord(k))
        return int(output,2)

    def __repr__(self):
        output = ''
        for k in self.bitset:
            try:
                output += bin(k)
            except:
                output += bin(ord(k))
            output += '//'
        return output


f = open('../test/readoutboard-emulator/resources/example-data/d20220708_rb_1.dat', 'rb')

# the events start with 0xaaaa and end with 0x5555

# single bytes for this
head = 0xaa
tail = 0x55


binary_events = []
data = f.read()

def count_2byte_markers(data, marker, return_indices=False):
    total = 0
    indices = []
    datalen = len(data)
    for i, k in tqdm.tqdm(enumerate(data), total=datalen):
        if k == marker:
            if i <= datalen - 1:
                if data[i+1] == marker:
                    total +=1 
                    if return_indices:
                        indices.append(i)
    if return_indices:
        return total, indices
    return total

############################################

def get_event_start_stop(start_indices, stop_indices):

    event_brackets = []
    i,k = 0,0
    start = start_indices[i]
    stop  = stop_indices[k]
    # first figure out if they are in the right order
    if start >= stop:
        while (start >= stop):
            i += 1
            stop = stop_indices[i]
        # truncate the stop indices
        stop_indices = stop_indices[i:]
    # check the last entries
    last_start, last_stop = start_indices[-1], stop_indices[-1]
    if last_start >= last_stop:
        tmpindex = -1
        while last_start >=last_stop:
            tmpindex -= 1
            last_start = start_indices[tmpindex]
        # truncate again
        start_indices = start_indices[:tmpindex]
    print (len(start_indices))
    print (len(stop_indices))
    print (start_indices[0], start_indices[-1])
    print (stop_indices[0], stop_indices[-1])
    brackets = []
    # make sure for each start there is one stop

    #assert (len(start_indices) == len(stop_indices))
    
    return (zip (start_indices, stop_indices))

############################################

def get_events(eventstream):

    # how many events can be in there?
    max_events_in_file = int(float(len(eventstream))/BLOBEVTSIZE)
    print (f'Searching through {max_events_in_file } possible events!')
    progress = Progress()
    task = progress.add_task("Searching for events...", total=len(eventstream))

    finished = False
    current_index = 0
    binary_events = []

    n_events_found = 0
    NITER = 0
    while not finished:
        assert current_index >= 0
        head_index = search_for_head(eventstream[current_index:])
        if head_index == -1:
            finished = True
        # make an educated guess where the tail is
        tail_index = search_for_tail(eventstream[current_index + BLOBEVTSIZE - 5:])
        #print ('what is here', eventstream[current_index: current_index+2])        
        event_size = tail_index - head_index
        if event_size < 0:
            #print ('event < 0')
            current_index += 1
            continue
        #print (event_size, BLOBEVTSIZE)
        if event_size != BLOBEVTSIZE:
            #print ('error, event corrupt1')
            # currently the events all have the same size
            current_index += event_size
            continue
        # remember, tail index includes the tail!
        binary_events.append(eventstream[head_index: tail_index])
        #print ('lbe',len(binary_events))
        n_events_found += 1 
        progress.update(task, advance = n_events_found/max_events_in_file) 
        #print (tail_index - head_index, BLOBEVTSIZE)
        current_index += event_size
        print ('index', current_index)
        #time.sleep(1)
        #finished = True
        if tail_index == -1:
            finished = True

        NITER += 1
        if NITER > 900:
            print ('finished by NITER')
            finished = True

    return binary_events


############################################

def find_first_head(eventstream):
    for i,k in enumerate(eventstream):
        if k == head:
            if eventstream[i+1] == head:
                print (BitSet(eventstream[i: i+2]))
                print (f'status : {BitSet(eventstream[i+2:i + 4])}')
                print (f'len    : {BitSet(eventstream[i+4:i+6])}')
                #print (f'len    : {BitSet(eventstream[i+4:i+6]).to_number()}')
                print (eventstream[i+4], eventstream[i+6])
                test = eventstream[i+BLOBEVTSIZE-5: i + BLOBEVTSIZE+ 5]
                test = [int(k) for k in test]
                print (test)
                print ('----')
                time.sleep(1)

############################################

# search for markers, byte by byte
def search_for_head(eventstream):
    #print ('sfh', len(eventstream)) 
    for l,m in enumerate(eventstream):
        if m == head:
            if eventstream[l+1] == head:
                return l
    return -1

############################################

def search_for_tail(eventstream):
    for l,m in enumerate(eventstream):
        if m == tail:
            if eventstream[l+1] == tail:
                return l+2
    return -1

raise

MAXINDEX = len(data)
index = 0

b_events = get_events(data)

print (len(b_events))
raise 
find_first_head(data)
raise

starts = count_2byte_markers(data, head, return_indices=True)
ends   = count_2byte_markers(data, tail, return_indices=True)

print (f'We found {starts[0]} start and  {ends[0]} end markers!')

event_brackets = get_event_start_stop(starts[1], ends[1])
event_sizes = [k[1] - k[0] for k in event_brackets]

h = d.factory.hist1d(event_sizes, 20)
h.line(filled=True, alpha=0.8)
p.show()

raise

print (MAXINDEX)

progress = Progress()
task = progress.add_task("Searching for events...", total=MAXINDEX)
while index < MAXINDEX:
    head = search_for_head(data[index:])
    if head > -1:
        print ('found head, searching for tail')
        tail = search_for_tail(data[head:])
        if tail > -1:
            binary_events.append(data[head:tail+1])
            index = tail
    print (index)
    progress.update(task, advance=float(index)/MAXINDEX)
    if tail == -1:
        break


print (len(binary_events))
raise
n_events = 0
for evtbinary in f.readlines():
    print (evtbinary)
    event = ptf.decode_blobevent([k for k in evtbinary], 0)
    print (event.event_ctr)
    print (event)
    print ('---')
    n_events += 1
    if n_events > 3:
        break
print (f'We read {n_events}')
