#! /bin/sh 
python -m build 
#auditwheel repair --plat manylinux_2_39_x86_64 dist/gondola_cxx-0.12.1-cp313-cp313-linux_x86_64.whl 
auditwheel repair dist/gondola_cxx-0.12.1-cp314-cp314-linux_x86_64.whl 
