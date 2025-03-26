#! /usr/bin/sh

# we assume we are in the scripts directory
# FIXME

cd ../../gaps_db
rm gaps_flight.db
rm -rf tof-db/migrations
python manage.py makemigrations
python manage.py migrate
cd ../resources/scripts/
./create_tof_db_flight.py --coordinates ../master-spreadsheet/tof-paddle-orientations-clean.xlsx --volid-map ../master-spreadsheet/paddleid_vs_volid.json --cable-map ../master-spreadsheet/Jeff_paddle_cable.json --create-all-tables ../master-spreadsheet/GAPS_Channel_mapping_v2.1.xlsx
./create_trk_db_flight.py --coordinates ../master-spreadsheet/tracker-from-sd.json --pedestals ../tracker-calibration/pedestal_LDB_9December.txt 
