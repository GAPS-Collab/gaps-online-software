"""
Compare the column densities of gondola_column_density.py
and root_column_density.C. The air is left out of the
gondola values, since ROOT (as crane) skips the world
volume. For the tracks with the largest differences, the
path lengths in the volumes which differ most are shown.
"""

import argparse
from collections import defaultdict

import numpy as np

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('tracks', help='tracks from make_tracks.py')
parser.add_argument('outdir', help='directory with the output of both')
parser.add_argument('--worst', type=int, default=5, help='number of tracks to show in detail')
args = parser.parse_args()

ours   = np.loadtxt(f'{args.outdir}/ours.txt')
root   = np.loadtxt(f'{args.outdir}/root.txt', ndmin=2)
n      = len(root)
ours   = ours[:n]
tracks = np.loadtxt(args.tracks, ndmin=2)[:n]
x_air, x_ours = ours[:, 1], ours[:, 2]
x_root, x_fine, dt = root[:, 1], root[:, 2], root[:, 3]
length = np.linalg.norm(tracks[:, 3:] - tracks[:, :3], axis=1)

print(f'{n} tracks, mean column density without air: gondola {x_ours.mean():.4f}, ROOT {x_root.mean():.4f} g/cm2')
diff = x_ours - x_root
print(f'gondola - ROOT (0.1 mm)  : mean {diff.mean():+.2e}, rms {diff.std():.2e}, max |.| {np.abs(diff).max():.2e} g/cm2')
large = x_root > 0.1
if large.any():
    rel = np.abs(diff[large]/x_root[large])
    print(f'  relative ({large.sum()} tracks with X > 0.1 g/cm2): median {np.median(rel):.2e}, '
          f'99% {np.quantile(rel, 0.99):.2e}, max {rel.max():.2e}')
fine = ~np.isnan(x_fine)
if fine.any():
    d = x_ours[fine] - x_fine[fine]
    print(f'gondola - ROOT (0.01 mm) : mean {d.mean():+.2e}, rms {d.std():.2e}, max |.| {np.abs(d).max():.2e} g/cm2 '
          f'({fine.sum()} tracks, with 0.1 mm on the same: rms {diff[fine].std():.2e})')
print(f'the air adds {np.mean(x_air - x_ours):.2e} g/cm2 on average')
print(f'ROOT with 0.1 mm steps: {1e3*dt.mean():.1f} ms per track, {1e6*dt.sum()/(length.sum()/0.1):.2f} us per step')

l_ours, l_root = defaultdict(float), defaultdict(float)
for k, v, l in np.loadtxt(f'{args.outdir}/ours_volumes.txt', ndmin=2):
    l_ours[(int(k), int(v))] += l
for k, v, l in np.loadtxt(f'{args.outdir}/root_volumes.txt', ndmin=2):
    l_root[(int(k), int(v))] += l
for k in np.argsort(-np.abs(diff))[:args.worst]:
    print(f'track {k}: length {length[k]:.0f} mm, gondola {x_ours[k]:.4f}, ROOT {x_root[k]:.4f}, '
          f'ROOT 0.01 mm {x_fine[k]:.4f} g/cm2')
    keys = {v for (kk, v) in list(l_ours) + list(l_root) if kk == k}
    for _, v in sorted(((abs(l_ours.get((k, v), 0) - l_root.get((k, v), 0)), v) for v in keys), reverse=True)[:4]:
        print(f'    copynumber {v}: gondola {l_ours.get((k, v), 0):.3f} mm, ROOT {l_root.get((k, v), 0):.3f} mm')
