#! /bin/sh
#
# Render diagram with mermaid-cli
#

mmdc -i dataflow.mmd -o dataflow.pdf #-b transparent
mmdc -i readoutboard-soft.mmd -o readoutboard-soft.pdf #-b transparent
#mupdf readoutboard-soft.pdf
