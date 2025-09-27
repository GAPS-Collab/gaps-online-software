#! /bin/sh

rye init 
rye add pelican markdown
rye run pelican-themes --symlink ../../resources/extern/blue-penguin-dark 
rye run pelican pages -s pelicanconf.py -t blue-penguin-dark
