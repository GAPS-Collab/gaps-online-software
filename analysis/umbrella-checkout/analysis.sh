#! /usr/bin/sh

poetry run ./umbrella_checkout.py --plotdir content/images/ -c /data0/gaps/csbf/csbf-data/calib/20240525 /data0/gaps/csbf/csbf-data/11
pelican-themes -i pelican-twitchy
pelican content

