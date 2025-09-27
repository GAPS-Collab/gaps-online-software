#! /bin/sh

rye init 
rye add pelican markdown
rye sync 

mkdir pages/static
cp ../../resources/assets/lelewaa.png pages/static/ 
cp ../../resources/assets/pakii.webp pages/static/ 
cp ../../resources/assets/GAPSLOGO_2023_small.png pages/static/ 
rye run pelican-themes -v --install ../../resources/extern/blue-penguin-dark 
rye run pelican pages -s pelicanconf.py -t blue-penguin-dark

