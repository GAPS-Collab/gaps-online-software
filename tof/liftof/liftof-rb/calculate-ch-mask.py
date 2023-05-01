#! /usr/bin/env python

import sys

mask = sys.argv[1]
mask = "0b" + mask
print (int(mask, base=2))
