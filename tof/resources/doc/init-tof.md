# FOT initialization protocol

_The relevant liftof code is manged by systemd. Status of running 
applications can always be checked with `sudo systemd status <name>`_

To monitor all steps, open `/home/tof/tof-moni/litof-tui` (add `--from-telemetry` 
if we are not hooked up to the ethernet and `--alert-manifest alert-manifest` if you
want to get alerts)

1) Ensure CAT is operational, ssh into the box
2) stop the data taking service `sudo systemctl stop liftof`
2) `cd $HOME/bin`
3) `./liftof-status status` - note down missing RBs, 
   if willing powercycle some of the RATs. 
   _To powercycle RATs, they have to be soft-shutdowned first.
   To soft-shutdown a RAT `./liftof-shutdown <rat-id>`_
3) delete everything superfluos in `/tofdata` and `/tofdata/calib`. _Keep the link /tofdata/calib/latest/`_ - it is managed automatically anyway.
4) make sure the file `/tofdata/10000` exists - run ids for flight will
   start with 10001 then
5) run a RBCalibration `./liftof-cc -c ../staging/init/liftof-config.toml` calibration
   - this takes about 5mins, so it is ok to get a quick coffee. You can copy the data 
   from /tofdata/calib/latest to your local machine if you plan to look at it.
6) delete liftof (lifof-cc, liftof-scheduler) related logfiles in `$HOME/log/`
7) delete logs on the rbs (`$HOME/bin/delete-rb-logs`)
8) OPTIONAL : make sure the latest software is deployed on the rbs:
   `cd $HOME/bootstrap-tof`
   `./bs-liftof-rb.sh`
   For that step you have to make sure to use the correct liftof version you want
9) ensure `liftof-scheduler` service is running, this is needed to interact with 
   commands received from the flight computer
   `sudo systemctl restart liftof-scheduler`. 
10) verify /home/gaps/staging/liftof-config.toml,
    /home/gaps/next/liftof-config.toml and /home/gaps/default/liftof-config.toml
    that these are basically the same file and actually that what you want.
11) `sudo systemctl start liftof` Start a run and we are ready to go!
