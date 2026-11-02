"""
Column densities with gondola (ray casting) for the tracks
of make_tracks.py.

Writes <outdir>/ours.txt (track, column density with air,
without air, number of crossings) and
<outdir>/ours_volumes.txt (track, copy number, path length
in mm). Prints the timing, rejected volumes and overlapping
placements along the tracks.
"""

import argparse
import time
from collections import Counter

import numpy as np

from gondola.geometry import Geometry

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('gdml', help='GDML geometry file')
parser.add_argument('tracks', help='tracks from make_tracks.py')
parser.add_argument('outdir', help='directory for the output files')
parser.add_argument('--cache', default=None, help='binary cache of the parsed geometry')
args = parser.parse_args()

geo    = Geometry.load(args.gdml, args.cache)
tracks = np.loadtxt(args.tracks)
points = [(tuple(t[:3]), tuple(t[3:])) for t in tracks]

start = time.perf_counter()
geo.column_density((0, 0, 0), (0, 0, 1))
print(f'Raycaster built in {time.perf_counter() - start:.2f} s')

start = time.perf_counter()
for a, b in points:
    geo.column_density(a, b)
dt = time.perf_counter() - start
print(f'{len(points)} tracks in {dt:.3f} s ({1e6*dt/len(points):.1f} us per track, including python)')

results, volumes = [], []
n_rejected = 0
overlaps   = Counter()
def name(copynumber):
    volume = geo.volumes[geo.placement(copynumber).volume]
    return f"{volume.name.split('0x')[0]} ({geo.materials[volume.material].name.split('0x')[0]})"

for k, (a, b) in enumerate(points):
    path  = geo.trace(a, b)
    noair = sum(c.column_density for c in path.crossings if c.copynumber is not None)
    results.append((k, path.column_density, noair, len(path.crossings)))
    n_rejected += len(path.rejected)
    for c1, c2, _ in path.overlaps():
        overlaps[tuple(sorted([name(c1), name(c2)]))] += 1
    for c in path.crossings:
        if c.copynumber is not None:
            volumes.append((k, c.copynumber, c.length))

np.savetxt(f'{args.outdir}/ours.txt', np.array(results), fmt='%d %.9g %.9g %d')
np.savetxt(f'{args.outdir}/ours_volumes.txt', np.array(volumes), fmt='%d %d %.9g')
print(f'rejected volumes: {n_rejected}, overlapping placements along {sum(overlaps.values())} tracks')
for pair, n in overlaps.most_common():
    print(f'  {n:4d} {pair[0]} <-> {pair[1]}')
