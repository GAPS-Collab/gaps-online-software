#! /usr/bin/env python

import hjson

f = open('paddle-id-to-volume-id.json', 'w')

pids = range(1,161)
mapping = dict()
for k in pids:
  # 1-12 are in the tof cube, 
  # which is 1 (tof) 1 (inner) 0 (+z)
  if k <= 12: 
    vid = 110000000 + (k-1)*100
    mapping[k] = vid
    continue
  elif k <= 24:
    vid = 111000000 + (k - 12 -1)*100
    mapping[k] = vid
    continue
hjson.dump(mapping, f)
