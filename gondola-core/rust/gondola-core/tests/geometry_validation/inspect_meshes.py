"""
Inspect the solids of a GDML geometry: are the tessellated
solids closed and consistently oriented, which boolean and
primitive solids are used, and how many placements there
are per material.
"""

import argparse
from collections import Counter

import numpy as np

from gondola.geometry import Geometry, SolidKind

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('gdml', help='GDML geometry file')
parser.add_argument('--cache', default=None, help='binary cache of the parsed geometry')
args = parser.parse_args()

geo   = Geometry.load(args.gdml, args.cache)
stats = Counter()
open_solids = []
for s in geo.solids:
    if s.kind != SolidKind.Tessellated:
        continue
    v = s.vertices
    t = s.triangles.astype(np.int64)
    a, b, c = v[t[:, 0]], v[t[:, 1]], v[t[:, 2]]
    stats['degenerate triangles'] += int((np.linalg.norm(np.cross(b - a, c - a), axis=1) < 1e-12).sum())
    edges = np.concatenate([t[:, [0, 1]], t[:, [1, 2]], t[:, [2, 0]]])
    _, n_undirected = np.unique(np.sort(edges, axis=1), axis=0, return_counts=True)
    _, n_directed   = np.unique(edges, axis=0, return_counts=True)
    if (n_undirected > 2).any():
        stats['non-manifold'] += 1
    elif (n_undirected == 1).any():
        stats['open'] += 1
        open_solids.append((s.name, int((n_undirected == 1).sum())))
    else:
        stats['closed'] += 1
    if (n_directed > 1).any():
        stats['inconsistently oriented'] += 1
    # the enclosed volume is negative for inward normals
    if np.einsum('ij,ij->i', a, np.cross(b, c)).sum() < 0:
        stats['inward normals'] += 1
print('tessellated solids:', dict(stats))
for name, n in open_solids:
    print(f'  open: {name} ({n} boundary edges)')

for s in geo.solids:
    if s.kind in (SolidKind.Union, SolidKind.Intersection, SolidKind.Subtraction):
        first, second = s.operands
        print(f'{s.kind}: {s.name} = {geo.solids[first].name} ({geo.solids[first].kind}), '
              f'{geo.solids[second].name} ({geo.solids[second].kind})')

placements = Counter()
for p in geo.world_placements()[1:]:
    placements[geo.volumes[p.volume].material] += 1
for m, n in placements.items():
    print(f'{geo.materials[m].name:<32} {geo.materials[m].density:<10g} g/cm3 {n:>6} placements')
