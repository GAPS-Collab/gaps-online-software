// Column densities with ROOT, as calculated in crane
// (GGeometryTools::ColumnDensity): the track is sampled in
// steps, the volume at each step is found with FindNode,
// the volume id is the copy number, and steps in the world
// (id 0) or in id 1 are skipped.
//
// root -l -b -q 'root_column_density.C+("geometry.v26.08.gdml", "tracks.txt", "out", 100)'
//
// Writes out/root.txt (track, column density with 0.1 mm
// steps, with 0.01 mm steps for the first nfine tracks,
// time for the 0.1 mm steps in s) and out/root_volumes.txt
// (track, copy number, path length in mm with 0.1 mm steps).
// This file is part of gaps-online-software and published
// under the GPLv3 license

#include <chrono>
#include <cmath>
#include <fstream>
#include <map>
#include <string>

#include "TError.h"
#include "TGeoManager.h"
#include "TGeoMaterial.h"
#include "TGeoNode.h"
#include "TGeoVolume.h"

double column_density(const double* a, const double* b, double step, std::map<int, double>* lengths) {
  double d[3] = {b[0] - a[0], b[1] - a[1], b[2] - a[2]};
  double L    = sqrt(d[0]*d[0] + d[1]*d[1] + d[2]*d[2]);
  double x    = 0;
  for (double s = 0; s < L; s += step) {
    TGeoNode* node = gGeoManager->FindNode(a[0] + s*d[0]/L, a[1] + s*d[1]/L, a[2] + s*d[2]/L);
    if (!node) continue;
    int vid = node->GetNumber();
    if (vid == 0 || vid == 1) continue;
    // step in mm, density in g/cm3 -> g/cm2
    x += step*node->GetVolume()->GetMaterial()->GetDensity()/10;
    if (lengths) (*lengths)[vid] += step;
  }
  return x;
}

void root_column_density(std::string gdml, std::string tracks, std::string outdir, int nfine = 100) {
  // as in crane (GGeometry.cc), lengths in mm
  TGeoManager::LockDefaultUnits(false);
  TGeoManager::SetDefaultUnits(TGeoManager::EDefaultUnits::kG4Units);
  TGeoManager::LockDefaultUnits(true);
  gErrorIgnoreLevel = kWarning;
  TGeoManager::SetVerboseLevel(0);
  TGeoManager::Import(gdml.c_str());

  std::ifstream in(tracks);
  std::ofstream out(outdir + "/root.txt"), volumes(outdir + "/root_volumes.txt");
  double a[3], b[3];
  int    k     = 0;
  double total = 0;
  while (in >> a[0] >> a[1] >> a[2] >> b[0] >> b[1] >> b[2]) {
    std::map<int, double> lengths;
    auto   start = std::chrono::steady_clock::now();
    double x     = column_density(a, b, 0.1, &lengths);
    double dt    = std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count();
    total += dt;
    double x_fine = k < nfine ? column_density(a, b, 0.01, nullptr) : NAN;
    out << k << " " << x << " " << x_fine << " " << dt << "\n";
    for (auto& l : lengths) volumes << k << " " << l.first << " " << l.second << "\n";
    k++;
  }
  printf("%d tracks, %.1f s for the column densities with 0.1 mm steps\n", k, total);
}
