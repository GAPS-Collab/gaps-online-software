#! /usr/bin/sh

# we assume we are in the scripts directory
# FIXME

# delete the whole shmagoigl
cd ../../gaps_db
rm gaps_flight.db
rm -rf tof-db/migrations
python manage.py makemigrations
python manage.py migrate

cd ../resources/scripts/
./create_tof_db_flight.py --coordinates ../master-spreadsheet/tof-paddle-orientations-clean.xlsx --volid-map ../master-spreadsheet/paddleid_vs_volid.json --cable-map ../master-spreadsheet/Jeff_paddle_cable.json --create-all-tables ../master-spreadsheet/GAPS_Channel_mapping_v2.1.xlsx
#./create_trk_db_flight.py --coordinates ../master-spreadsheet/tracker-from-sd.json --pedestals ../tracker-calibration/pedestal_LDB_9December.txt --tracker-mask  ../tracker-calibration/SiLi_active_chs_241207.txt --transfer-fns ../tracker-calibration/TransferFnMcMurdoPoly.txt 
./create_trk_db_flight.py --coordinates ../master-spreadsheet/tracker-from-sd.json --transfer-fns ../tracker-calibration/TransferFnMcMurdoPoly.txt

# masks 
./add_trk_mask_to_db.py --mask-file ../tracker-calibration/SiLi_active_chs_241207.txt
# pedestals 
./add_trk_ped_to_db.py --pedestal-file ../tracker-calibration/pedestal_LDB_9December.txt 
# will currently add 3 pulse files
TRK_GAIN_FILE=../tracker-calibration/SiLi-gains.txt 
./add_trk_pls_to_db.py --gain-file $TRK_GAIN_FILE --utc-start 0 --utc-stop 1733951270 --pulse-file ../tracker-calibration/SiLi-pulsed_channels_1211.txt 
./add_trk_pls_to_db.py --gain-file $TRK_GAIN_FILE --utc-start 1733951270 --utc-stop 1734031538 --pulse-file ../tracker-calibration/SiLi-pulsed_channels_1212.txt 
./add_trk_pls_to_db.py --gain-file $TRK_GAIN_FILE --utc-start 1734031538 --utc-stop 9999999999 --pulse-file ../tracker-calibration/SiLi-pulsed_channels_1213.txt 

