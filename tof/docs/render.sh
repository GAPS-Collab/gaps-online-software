#! /bin/sh
#
# Render diagram with mermaid-cli
#

mmdc -i dataflow.mmd -o dataflow.pdf #-b transparent
mupdf dataflow.pdf
