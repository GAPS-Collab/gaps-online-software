#! /bin/sh

compile_and_deploy_target() {
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin $1 --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/$1 $2:~/bin/ 
}

# first delete everything, since there might be remains of a previously issued cargo check
rm -rf ../target/armv7-unknown*

# RBs at SSL
SSL_RB="tof-rb01 tof-rb02 tof-rb03 tof-rb04 tof-rb07 tof-rb08 tof-rb09 tof-rb11 tof-rb12 tof-rb13 tof-rb14 tof-rb15 tof-rb16 tof-rb17 tof-rb18 tof-rb19 tof-rb20 tof-rb22 tof-rb24 tof-rb25 tof-rb26 tof-rb27"

for rb in `echo $SSL_RB`; 
  do echo $rb;
  scp liftof.service $rb:bin/;
done

#compile_and_deploy_target liftof-rb ssl-tof-computer
#compile_and_deploy_target liftof-rb tof-rb01

