"""
The GAPS detector geometry, read from GDML.

The geometry is parsed by gondola-core (rust). Since the
GDML file is large (~500 MB), the parsed geometry can be
cached in a binary file, which loads in well under a second:

    geo = Geometry.load('geometry.v26.08.gdml', 'geometry.v26.08.bin')

Placements can be looked up by their Geant4 copy number
(copynumber in the GDML file):

    p = geo.placement(300000000)

The column density (in g/cm2) along a straight track, or
along a tracklet, is calculated by ray casting:

    x    = geo.column_density((0, 0, -100), (0, 0, 2000))
    x    = geo.tracklet_column_density(event.primary)
    path = geo.trace((0, 0, -100), (0, 0, 2000))

All lengths are in mm, angles in rad and densities in g/cm3.
"""

import argparse
import time
from collections import Counter

from . import _gondola_core

Geometry = _gondola_core.geometry.Geometry
Geometry.__module__ = __name__
Geometry.__name__   = 'Geometry'

Material = _gondola_core.geometry.Material
Material.__module__ = __name__
Material.__name__   = 'Material'

Solid = _gondola_core.geometry.Solid
Solid.__module__ = __name__
Solid.__name__   = 'Solid'

SolidKind = _gondola_core.geometry.SolidKind
SolidKind.__module__ = __name__
SolidKind.__name__   = 'SolidKind'

Volume = _gondola_core.geometry.Volume
Volume.__module__ = __name__
Volume.__name__   = 'Volume'

Placement = _gondola_core.geometry.Placement
Placement.__module__ = __name__
Placement.__name__   = 'Placement'

WorldPlacement = _gondola_core.geometry.WorldPlacement
WorldPlacement.__module__ = __name__
WorldPlacement.__name__   = 'WorldPlacement'

Transform = _gondola_core.geometry.Transform
Transform.__module__ = __name__
Transform.__name__   = 'Transform'

Crossing = _gondola_core.geometry.Crossing
Crossing.__module__ = __name__
Crossing.__name__   = 'Crossing'

TrackPath = _gondola_core.geometry.TrackPath
TrackPath.__module__ = __name__
TrackPath.__name__   = 'TrackPath'

#------------------------------------------------

def print_summary(geo : Geometry) -> None:
    """
    Print an inventory of the geometry (materials,
    solids, volumes and placements)
    """
    kinds = Counter()
    n_tri, n_vert = 0, 0
    for s in geo.solids:
        kinds[s.kind] += 1
        n_tri  += s.n_triangles
        n_vert += s.n_vertices
    placements = geo.world_placements()
    max_depth  = max(p.depth for p in placements)

    print(f'Materials            : {len(geo.materials)}')
    for m in geo.materials:
        print(f'  {m.name:<40} {m.density:>12.6e} g/cm3')
    print(f'Solids               : {len(geo.solids)}')
    print(f'  tessellated        : {kinds[SolidKind.Tessellated]} ({n_tri} triangles, {n_vert} vertices)')
    print(f'  box / tube         : {kinds[SolidKind.Box]} / {kinds[SolidKind.Tube]}')
    print(f'  union / inter. / sub. : {kinds[SolidKind.Union]} / {kinds[SolidKind.Intersection]} / {kinds[SolidKind.Subtraction]}')
    print(f'Logical volumes      : {len(geo.volumes)}')
    print(f'Placed volumes       : {len(placements) - 1} (excluding the world, max. depth {max_depth})')
    world = geo.world_volume
    print(f'World volume         : {world.name} ({geo.materials[world.material].name}, {geo.density(geo.world)} g/cm3)')

#------------------------------------------------

def main():
    """
    Parse a GDML file (or read a geometry cache) and print
    an inventory of the geometry.

    Installed as the gdml-summary command.
    """
    parser = argparse.ArgumentParser(description=main.__doc__)
    parser.add_argument('gdml', help='GDML geometry file')
    parser.add_argument('cache', nargs='?', default=None,
                        help='Binary cache of the parsed geometry. It is used if it is newer than the GDML file, otherwise it gets (re)written.')
    args = parser.parse_args()

    start = time.perf_counter()
    geo   = Geometry.load(args.gdml, args.cache)
    print(f'Loaded geometry in {time.perf_counter() - start:.2f} s')
    print_summary(geo)
