#! /bin/sh 

uv run streamlit run gaze.py --server.headless true --server.port 8080 $@
