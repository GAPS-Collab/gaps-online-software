#! /bin/sh

compile_and_deploy_target() {
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin $1 --target=armv7-unknown-linux-musleabi --release && scp ../target/armv7-unknown-linux-musleabi/release/$1 $2:~/bin/ 
}

# first delete everything, since there might be remains of a previously issued cargo check
rm -rf ../target/armv7-unknown*

# RBs at SSL
SSL_RB="tof-rb01 tof-rb02 tof-rb03 tof-rb04 tof-rb07 tof-rb08 tof-rb09 tof-rb11 tof-rb12 tof-rb13 tof-rb14 tof-rb15 tof-rb17 tof-rb18 tof-rb19 tof-rb20"

for rb in `echo $SSL_RB`; 
  do echo $rb;
  scp liftof.service $rb:bin/;
done
#compile_and_deploy_target liftof-rb ucla-rb149
#for rb in ucla-rb00 ucla-rb01 ucla-rb02 ucla-rb03 ucla-rb04 ucla-rb07 ucla-rb08 ucla-rb09 ucla-rb10;
#for rb in tof-rb05 tof-rb06 tof-rb07 tof-rb08 tof-rb09 tof-rb10
#for rb in tof-rb52 tof-rb05 tof-rb06

#for rb in tof-rb01 tof-rb02 tof-rb03 tof-rb04 tof-rb07 tof-rb08 tof-rb09 tof-rb10 
#for rb in tof-rb10 #tof-rb22
#do compile_and_deploy_target liftof-rb $rb;
#done

#compile_and_deploy_target liftof-rb tof-rb30
#compile_and_deploy_target liftof-rb ssl-tof-computer
#compile_and_deploy_target liftof-rb tof-rb30
#compile_and_deploy_target liftof-rb tof-rb05
#com
#pile_and_deploy_target liftof-rb tof-rb06
#compile_and_deploy_target liftof-rb tof-rb01
#compile_and_deploy_target liftof-rb tof-rb02
#compile_and_deploy_target scan-uio-buffers tof-rb05
#compile_and_deploy_target debug-idle
#compile_and_deploy_target watch-buffer-fill

