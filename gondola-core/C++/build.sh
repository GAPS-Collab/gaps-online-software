#! /bin/sh

CURRENT_HOST=$(hostname)
TARGET_HOST="uhcra01.cra.lan"

if [ "$CURRENT_HOST" = "$TARGET_HOST" ]; then
    echo "BUILDING on uhcra!"
    # for building on uhcra 
    #alias python=/usr/bin/python3.11  
    source .venv/bin/activate 
    export _PYTHON_HOST_PLATFORM="manylinux_2_35_x86_64"  
    
    python -m build -w --no-isolation\
      -Ccmake.define.CMAKE_C_COMPILER=/usr/bin/gcc \
      -Ccmake.define.CMAKE_CXX_COMPILER=/usr/bin/g++\
      -Ccmake.define.Python_EXECUTABLE=/usr/bin/python3.11
else
    # it's probably valkyrie
    #python -m build 
    auditwheel repair dist/gondola_cxx-0.12.1-cp314-cp314-linux_x86_64.whl
fi



