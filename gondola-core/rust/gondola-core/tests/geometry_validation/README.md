# Validation of the geometry and the column density

Scripts used to validate `gondola.geometry` (see `GEOMETRY.md` in the
crate root). They need the GDML file, so they are not part of the
regular test suite. The Python scripts need the gondola Python
bindings (`maturin develop` or `build.sh`). If `import gondola` fails
outside Jupyter, set `MPLBACKEND=agg`.

```sh
GDML=/srv/gaps/crane/v26.08/resources/geometry/geometry.v26.08.gdml
CACHE=geometry.v26.08.bin
OUT=out; mkdir -p $OUT
```

## Ray casting through every mesh

The ignored integration test `tests/geometry_volume_check.rs` compares
the path length, integrated over a grid of parallel lines, with the
volume enclosed by each tessellated solid:

```sh
GAPS_GDML=$GDML GAPS_GDML_CACHE=$CACHE GAPS_VOLUME_CHECK_N=256 \
  cargo test --release --test geometry_volume_check -- --ignored --nocapture
```

## Column density compared with ROOT

```sh
python make_tracks.py $OUT/tracks.txt -n 1000
python gondola_column_density.py $GDML $OUT/tracks.txt $OUT --cache $CACHE
source /srv/root/root-6.40-patches-install/bin/thisroot.sh   # from bash
root -l -b -q "root_column_density.C+(\"$GDML\", \"$OUT/tracks.txt\", \"$OUT\", 100)"
python compare.py $OUT/tracks.txt $OUT
```

`root_column_density.C` reproduces crane's
`GGeometryTools::ColumnDensity`: 0.1 mm steps, `FindNode`, volume id =
copy number, and the world skipped. For the first 100 tracks it also
uses 0.01 mm steps. For 1000 tracks it runs for about a minute.

Results for geometry v26.08 (2026-11-01):

| Comparison | Mean difference | RMS | Max |
|---|---|---|---|
| gondola − ROOT, 0.1 mm steps | −0.0005 g/cm² | 0.040 g/cm² | 0.30 g/cm² |
| gondola − ROOT, 0.01 mm steps | +0.0004 g/cm² | 0.004 g/cm² | 0.022 g/cm² |

The mean column density was 14.55 g/cm² for both. Timing per track:

| Method | Time per track |
|---|---|
| gondola, including Python overhead | 28 µs |
| ROOT, 0.1 mm steps | 24 ms |

The differences shrink with ROOT's step length, so they come from
ROOT's sampling. `gondola_column_density.py` also lists overlapping
placements: in v26.08 these are small overlaps (≤ 0.2 mm) between
`PassiveTrackerModuleFrame` and the bottom-tracker mechanical parts.

## Mesh inspection

```sh
python inspect_meshes.py $GDML --cache $CACHE
```

This checks which tessellated solids are closed and consistently
oriented, and lists the boolean solids and the placements per
material.
