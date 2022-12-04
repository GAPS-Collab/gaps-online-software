/**
 *
 */

#include <iostream>
#include <sstream>
#include <cstddef>
#include <vector>

#include "BlobFileServer.h"


using namespace TOF;

int main() {

  std::vector<std::string> blobfiles;
  std::string blob_basename = "/srv/gaps/gfp-data/gaps-gfp/TOFsoftware/server/test/readoutboard-emulator/resources/example-data/d20220708_rb_";
  for (int k=1;k<5;k++) {
    blobfiles.push_back(blob_basename + std::to_string(k) + ".dat");
  }
  BlobFileServer source = BlobFileServer(blobfiles);
  source.SetFilesOnRepeat(false);
  source.Serve();
  return EXIT_SUCCESS;
}
