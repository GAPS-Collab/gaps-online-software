"""
Random straight tracks for the column density validation.

Half of the tracks are random segments in a box around the
instrument, the other half go from the top to the bottom
through it. Writes one track per line as
x0 y0 z0 x1 y1 z1 (in mm).
"""

import argparse

import numpy as np

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', help='output file, e.g. tracks.txt')
parser.add_argument('-n', type=int, default=1000, help='number of tracks')
parser.add_argument('--seed', type=int, default=20261101)
args = parser.parse_args()

rng  = np.random.default_rng(args.seed)
n1   = args.n//2
n2   = args.n - n1
a1   = rng.uniform([-1100, -1100, -300], [1100, 1100, 2300], (n1, 3))
b1   = rng.uniform([-1100, -1100, -300], [1100, 1100, 2300], (n1, 3))
a2   = np.column_stack([rng.uniform(-900, 900, (n2, 2)), np.full(n2,  2200.0)])
b2   = np.column_stack([rng.uniform(-900, 900, (n2, 2)), np.full(n2, -200.0)])
tracks = np.vstack([np.hstack([a1, b1]), np.hstack([a2, b2])])
np.savetxt(args.output, tracks, fmt='%.6f')

length = np.linalg.norm(tracks[:, 3:] - tracks[:, :3], axis=1)
print(f'{len(tracks)} tracks, mean length {length.mean():.1f} mm, '
      f'{length.sum()/0.1/1e6:.2f} M steps of 0.1 mm')
